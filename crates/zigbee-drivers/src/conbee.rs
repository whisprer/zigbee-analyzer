use zigbee_core::packet::RawPacket;
use zigbee_hal::{
    traits::ZigbeeCapture,
    capabilities::DeviceCapabilities,
    error::{HalError, HalResult},
};
use async_trait::async_trait;
use serialport::{SerialPort, SerialPortType};
use std::time::{Duration, SystemTime};
use std::io::{Read, Write};

/// ConBee/ConBee II/RaspBee/RaspBee II USB Zigbee Sniffer driver
pub struct ConBee {
    port: Option<Box<dyn SerialPort>>,
    port_name: String,
    channel: u8,
    active: bool,
    capabilities: DeviceCapabilities,
    buffer: Vec<u8>,
    variant: ConBeeVariant,
    firmware_version: Option<u32>,
}

// ConBee protocol constants (Dresden Elektronik proprietary)
const SLIP_END: u8 = 0xC0;
const SLIP_ESC: u8 = 0xDB;
const SLIP_ESC_END: u8 = 0xDC;
const SLIP_ESC_ESC: u8 = 0xDD;

// ConBee command types
const CMD_DEVICE_STATE: u8 = 0x07;
const CMD_VERSION: u8 = 0x0D;
const CMD_WRITE_PARAMETER: u8 = 0x05;
const CMD_READ_PARAMETER: u8 = 0x0A;
const CMD_APS_DATA_INDICATION: u8 = 0x17;
const CMD_APS_DATA_REQUEST: u8 = 0x12;
const CMD_MAC_POLL_INDICATION: u8 = 0x1C;

// Parameter IDs
const PARAM_MAC_ADDRESS: u8 = 0x01;
const PARAM_NWK_PANID: u8 = 0x05;
const PARAM_NWK_ADDR: u8 = 0x07;
const PARAM_NWK_EXTENDED_PANID: u8 = 0x08;
const PARAM_APS_DESIGNATED_COORDINATOR: u8 = 0x09;
const PARAM_CHANNEL_MASK: u8 = 0x0A;
const PARAM_APS_EXTENDED_PANID: u8 = 0x0B;
const PARAM_TRUST_CENTER_ADDRESS: u8 = 0x0E;
const PARAM_SECURITY_MODE: u8 = 0x10;
const PARAM_NETWORK_KEY: u8 = 0x18;
const PARAM_OPERATING_MODE: u8 = 0x25;

// Operating modes
const MODE_COORDINATOR: u8 = 0x00;
const MODE_ROUTER: u8 = 0x01;
const MODE_END_DEVICE: u8 = 0x02;

// Address modes
const ADDR_MODE_GROUP: u8 = 0x01;
const ADDR_MODE_NWK: u8 = 0x02;
const ADDR_MODE_IEEE: u8 = 0x03;

// USB VID/PID for ConBee devices
const DRESDEN_VID: u16 = 0x1cf1;
const CONBEE_PID: u16 = 0x0030;
const CONBEE_II_PID: u16 = 0x0031;
const RASPBEE_PID: u16 = 0x0028;
const RASPBEE_II_PID: u16 = 0x0029;

impl ConBee {
    pub fn new() -> HalResult<Self> {
        let (port_name, variant) = Self::find_device()?;
        
        let mut capabilities = DeviceCapabilities::basic_capture();
        capabilities.max_sample_rate = 400;
        capabilities.buffer_size = 1024;
        capabilities.hardware_timestamps = true;
        capabilities.promiscuous_mode = true;
        capabilities.energy_detection = true;
        capabilities.rssi_precision = 1;
        
        Ok(Self {
            port: None,
            port_name,
            channel: 11,
            active: false,
            capabilities,
            buffer: Vec::with_capacity(2048),
            variant,
            firmware_version: None,
        })
    }
    
    fn find_device() -> HalResult<(String, ConBeeVariant)> {
        let ports = serialport::available_ports()
            .map_err(|e| HalError::HardwareError(format!("Failed to enumerate ports: {}", e)))?;
        
        for port in ports {
            if let SerialPortType::UsbPort(info) = &port.port_type {
                if info.vid == DRESDEN_VID {
                    let variant = match info.pid {
                        CONBEE_PID => Some(ConBeeVariant::ConBee),
                        CONBEE_II_PID => Some(ConBeeVariant::ConBeeII),
                        RASPBEE_PID => Some(ConBeeVariant::RaspBee),
                        RASPBEE_II_PID => Some(ConBeeVariant::RaspBeeII),
                        _ => None,
                    };
                    
                    if let Some(v) = variant {
                        return Ok((port.port_name, v));
                    }
                }
            }
        }
        
        Err(HalError::DeviceNotFound)
    }
    
    pub fn new_with_port(port_name: String, variant: ConBeeVariant) -> HalResult<Self> {
        let mut capabilities = DeviceCapabilities::basic_capture();
        capabilities.max_sample_rate = 400;
        capabilities.buffer_size = 1024;
        capabilities.hardware_timestamps = true;
        capabilities.promiscuous_mode = true;
        capabilities.energy_detection = true;
        
        Ok(Self {
            port: None,
            port_name,
            channel: 11,
            active: false,
            capabilities,
            buffer: Vec::with_capacity(2048),
            variant,
            firmware_version: None,
        })
    }
    
    fn open_port(&mut self) -> HalResult<()> {
        let baud_rate = match self.variant {
            ConBeeVariant::ConBee | ConBeeVariant::RaspBee => 38400,
            ConBeeVariant::ConBeeII | ConBeeVariant::RaspBeeII => 115200,
        };
        
        let port = serialport::new(&self.port_name, baud_rate)
            .timeout(Duration::from_millis(100))
            .data_bits(serialport::DataBits::Eight)
            .stop_bits(serialport::StopBits::One)
            .parity(serialport::Parity::None)
            .flow_control(serialport::FlowControl::None)
            .open()
            .map_err(|e| HalError::SerialError(format!("Failed to open port: {}", e)))?;
        
        self.port = Some(port);
        Ok(())
    }
    
    fn slip_encode(&self, data: &[u8]) -> Vec<u8> {
        let mut encoded = vec![SLIP_END];
        
        for &byte in data {
            match byte {
                SLIP_END => {
                    encoded.push(SLIP_ESC);
                    encoded.push(SLIP_ESC_END);
                }
                SLIP_ESC => {
                    encoded.push(SLIP_ESC);
                    encoded.push(SLIP_ESC_ESC);
                }
                _ => {
                    encoded.push(byte);
                }
            }
        }
        
        encoded.push(SLIP_END);
        encoded
    }
    
    fn slip_decode(&self, data: &[u8]) -> Vec<u8> {
        let mut decoded = Vec::new();
        let mut escape_next = false;
        
        for &byte in data {
            if escape_next {
                match byte {
                    SLIP_ESC_END => decoded.push(SLIP_END),
                    SLIP_ESC_ESC => decoded.push(SLIP_ESC),
                    _ => decoded.push(byte),
                }
                escape_next = false;
            } else {
                match byte {
                    SLIP_END => {},
                    SLIP_ESC => escape_next = true,
                    _ => decoded.push(byte),
                }
            }
        }
        
        decoded
    }
    
    fn send_command(&mut self, cmd: u8, payload: &[u8]) -> HalResult<()> {
        let port = self.port.as_mut()
            .ok_or(HalError::NotInitialized)?;
        
        let seq = 0u8;
        let len = payload.len() as u16;
        
        let mut frame = vec![cmd, seq, 0x00, (len & 0xFF) as u8, ((len >> 8) & 0xFF) as u8];
        frame.extend_from_slice(payload);
        
        let encoded = self.slip_encode(&frame);
        
        port.write_all(&encoded)
            .map_err(|e| HalError::SerialError(format!("Write failed: {}", e)))?;
        
        port.flush()
            .map_err(|e| HalError::SerialError(format!("Flush failed: {}", e)))?;
        
        Ok(())
    }
    
    fn read_frame(&mut self, timeout: Duration) -> HalResult<Option<Frame>> {
        let port = self.port.as_mut()
            .ok_or(HalError::NotInitialized)?;
        
        port.set_timeout(timeout)
            .map_err(|e| HalError::SerialError(format!("Set timeout failed: {}", e)))?;
        
        let mut raw_buffer = Vec::new();
        let mut byte_buf = [0u8; 1];
        let mut in_frame = false;
        
        loop {
            match port.read_exact(&mut byte_buf) {
                Ok(_) => {
                    let byte = byte_buf[0];
                    
                    if byte == SLIP_END {
                        if in_frame && !raw_buffer.is_empty() {
                            break;
                        } else {
                            in_frame = true;
                            raw_buffer.clear();
                        }
                    } else if in_frame {
                        raw_buffer.push(byte);
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                    return Ok(None);
                }
                Err(e) => {
                    return Err(HalError::SerialError(format!("Read failed: {}", e)));
                }
            }
            
            if raw_buffer.len() > 2048 {
                return Err(HalError::InvalidPacket("Frame too large".to_string()));
            }
        }
        
        let decoded = self.slip_decode(&raw_buffer);
        
        if decoded.len() < 5 {
            return Err(HalError::InvalidPacket("Frame too short".to_string()));
        }
        
        let cmd = decoded[0];
        let seq = decoded[1];
        let status = decoded[2];
        let len = decoded[3] as usize | ((decoded[4] as usize) << 8);
        
        if decoded.len() < 5 + len {
            return Err(HalError::InvalidPacket("Incomplete frame data".to_string()));
        }
        
        let data = decoded[5..5 + len].to_vec();
        
        Ok(Some(Frame { cmd, seq, status, data }))
    }
    
    async fn query_version(&mut self) -> HalResult<()> {
        self.send_command(CMD_VERSION, &[])?;
        
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        match self.read_frame(Duration::from_millis(500))? {
            Some(frame) => {
                if frame.cmd == CMD_VERSION && frame.data.len() >= 4 {
                    let version = u32::from_le_bytes([
                        frame.data[0],
                        frame.data[1],
                        frame.data[2],
                        frame.data[3],
                    ]);
                    self.firmware_version = Some(version);
                }
                Ok(())
            }
            None => {
                Err(HalError::HardwareError("No version response".to_string()))
            }
        }
    }
    
    async fn set_channel_mask(&mut self, channel: u8) -> HalResult<()> {
        let channel_mask: u32 = 1 << channel;
        
        let mut payload = vec![PARAM_CHANNEL_MASK];
        payload.extend_from_slice(&channel_mask.to_le_bytes());
        
        self.send_command(CMD_WRITE_PARAMETER, &payload)?;
        
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        Ok(())
    }
    
    /// Parse APS Data Indication into raw 802.15.4 packet
    fn parse_aps_indication(&self, data: &[u8]) -> HalResult<RawPacket> {
        if data.len() < 2 {
            return Err(HalError::InvalidPacket("APS indication too short".to_string()));
        }
        
        let mut offset = 0;
        
        // Device state (1 byte)
        let _device_state = data[offset];
        offset += 1;
        
        // Destination address mode (1 byte)
        let dst_addr_mode = data[offset];
        offset += 1;
        
        // Destination address (variable length based on mode)
        let dst_addr_len = match dst_addr_mode {
            ADDR_MODE_GROUP => 2,
            ADDR_MODE_NWK => 2,
            ADDR_MODE_IEEE => 8,
            _ => 0,
        };
        
        if data.len() < offset + dst_addr_len {
            return Err(HalError::InvalidPacket("Insufficient data for dst addr".to_string()));
        }
        
        let _dst_addr = &data[offset..offset + dst_addr_len];
        offset += dst_addr_len;
        
        // Destination endpoint (1 byte) - only if not group mode
        if dst_addr_mode != ADDR_MODE_GROUP {
            if data.len() < offset + 1 {
                return Err(HalError::InvalidPacket("No dst endpoint".to_string()));
            }
            let _dst_endpoint = data[offset];
            offset += 1;
        }
        
        // Source address mode (1 byte)
        if data.len() < offset + 1 {
            return Err(HalError::InvalidPacket("No src addr mode".to_string()));
        }
        let src_addr_mode = data[offset];
        offset += 1;
        
        // Source address (variable length)
        let src_addr_len = match src_addr_mode {
            ADDR_MODE_GROUP => 2,
            ADDR_MODE_NWK => 2,
            ADDR_MODE_IEEE => 8,
            _ => 0,
        };
        
        if data.len() < offset + src_addr_len {
            return Err(HalError::InvalidPacket("Insufficient data for src addr".to_string()));
        }
        
        let _src_addr = &data[offset..offset + src_addr_len];
        offset += src_addr_len;
        
        // Source endpoint (1 byte)
        if data.len() < offset + 1 {
            return Err(HalError::InvalidPacket("No src endpoint".to_string()));
        }
        let _src_endpoint = data[offset];
        offset += 1;
        
        // Profile ID (2 bytes)
        if data.len() < offset + 2 {
            return Err(HalError::InvalidPacket("No profile ID".to_string()));
        }
        let _profile_id = u16::from_le_bytes([data[offset], data[offset + 1]]);
        offset += 2;
        
        // Cluster ID (2 bytes)
        if data.len() < offset + 2 {
            return Err(HalError::InvalidPacket("No cluster ID".to_string()));
        }
        let _cluster_id = u16::from_le_bytes([data[offset], data[offset + 1]]);
        offset += 2;
        
        // ASDU length (2 bytes)
        if data.len() < offset + 2 {
            return Err(HalError::InvalidPacket("No ASDU length".to_string()));
        }
        let asdu_len = u16::from_le_bytes([data[offset], data[offset + 1]]) as usize;
        offset += 2;
        
        // ASDU payload (variable)
        if data.len() < offset + asdu_len {
            return Err(HalError::InvalidPacket("Insufficient ASDU data".to_string()));
        }
        let asdu = &data[offset..offset + asdu_len];
        offset += asdu_len;
        
        // LQI (1 byte)
        if data.len() < offset + 1 {
            return Err(HalError::InvalidPacket("No LQI".to_string()));
        }
        let lqi = data[offset];
        offset += 1;
        
        // RSSI (1 byte) - signed
        if data.len() < offset + 1 {
            return Err(HalError::InvalidPacket("No RSSI".to_string()));
        }
        let rssi = data[offset] as i8;
        
        // Now we need to reconstruct the 802.15.4 MAC frame
        // The ASDU contains the NWK/APS payload, but we need the full MAC frame
        // We'll build it from the information we have
        
        let mut mac_frame = Vec::new();
        
        // Frame Control Field (2 bytes)
        // Frame Type: Data (0b001)
        // Security: depends on if encryption was used
        // Frame Pending: 0
        // Ack Request: typically 1
        // PAN ID Compression: depends on addressing
        // Dest Addr Mode: based on dst_addr_mode
        // Frame Version: 0b00 (2003)
        // Src Addr Mode: based on src_addr_mode
        
        let frame_type = 0b001; // Data frame
        let security = 0; // We don't know from APS indication
        let frame_pending = 0;
        let ack_request = 1;
        let pan_id_compression = if dst_addr_mode == src_addr_mode { 1 } else { 0 };
        
        let dst_addr_mode_fcf = match dst_addr_mode {
            ADDR_MODE_NWK => 0b10,
            ADDR_MODE_IEEE => 0b11,
            _ => 0b00,
        };
        
        let src_addr_mode_fcf = match src_addr_mode {
            ADDR_MODE_NWK => 0b10,
            ADDR_MODE_IEEE => 0b11,
            _ => 0b00,
        };
        
        let fcf_low = (frame_type & 0b111) | 
                      ((security & 1) << 3) |
                      ((frame_pending & 1) << 4) |
                      ((ack_request & 1) << 5) |
                      ((pan_id_compression & 1) << 6);
        
        let fcf_high = (dst_addr_mode_fcf << 2) | 
                       (src_addr_mode_fcf << 6);
        
        mac_frame.push(fcf_low);
        mac_frame.push(fcf_high);
        
        // Sequence number (1 byte) - we don't have it, use 0
        mac_frame.push(0);
        
        // Destination PAN ID (2 bytes) - use default for now
        mac_frame.extend_from_slice(&[0xFF, 0xFF]);
        
        // Destination address
        mac_frame.extend_from_slice(_dst_addr);
        
        // Source PAN ID (if not compressed)
        if pan_id_compression == 0 {
            mac_frame.extend_from_slice(&[0xFF, 0xFF]);
        }
        
        // Source address
        mac_frame.extend_from_slice(_src_addr);
        
        // Payload (the ASDU is the NWK payload)
        mac_frame.extend_from_slice(asdu);
        
        // FCS (2 bytes) - we'll calculate it
        let fcs = self.calculate_fcs(&mac_frame);
        mac_frame.extend_from_slice(&fcs.to_le_bytes());
        
        Ok(RawPacket {
            timestamp: SystemTime::now(),
            channel: self.channel,
            rssi,
            lqi,
            data: mac_frame,
        })
    }
    
    /// Calculate FCS (Frame Check Sequence) using CRC-16-CCITT
    fn calculate_fcs(&self, data: &[u8]) -> u16 {
        let mut crc: u16 = 0;
        
        for &byte in data {
            crc ^= (byte as u16) << 8;
            for _ in 0..8 {
                if crc & 0x8000 != 0 {
                    crc = (crc << 1) ^ 0x1021;
                } else {
                    crc <<= 1;
                }
            }
        }
        
        crc
    }
}

#[async_trait]
impl ZigbeeCapture for ConBee {
    async fn initialize(&mut self) -> HalResult<()> {
        self.open_port()?;
        
        tokio::time::sleep(Duration::from_millis(200)).await;
        
        self.query_version().await?;
        
        self.set_channel_mask(self.channel).await?;
        
        self.active = true;
        Ok(())
    }
    
    async fn set_channel(&mut self, channel: u8) -> HalResult<()> {
        if !(11..=26).contains(&channel) {
            return Err(HalError::InvalidChannel(channel));
        }
        
        self.set_channel_mask(channel).await?;
        self.channel = channel;
        
        Ok(())
    }
    
    fn get_channel(&self) -> HalResult<u8> {
        Ok(self.channel)
    }
    
    async fn capture_packet(&mut self) -> HalResult<RawPacket> {
        if !self.active {
            return Err(HalError::NotInitialized);
        }
        
        loop {
            match self.read_frame(Duration::from_secs(5))? {
                Some(frame) => {
                    if frame.cmd == CMD_APS_DATA_INDICATION {
                        return self.parse_aps_indication(&frame.data);
                    }
                }
                None => {
                    return Err(HalError::Timeout);
                }
            }
        }
    }
    
    fn try_capture_packet(&mut self) -> HalResult<Option<RawPacket>> {
        if !self.active {
            return Err(HalError::NotInitialized);
        }
        
        match self.read_frame(Duration::from_millis(1))? {
            Some(frame) => {
                if frame.cmd == CMD_APS_DATA_INDICATION {
                    Ok(Some(self.parse_aps_indication(&frame.data)?))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }
    
    fn capabilities(&self) -> &DeviceCapabilities {
        &self.capabilities
    }
    
    fn device_name(&self) -> &str {
        match self.variant {
            ConBeeVariant::ConBee => "ConBee USB Dongle",
            ConBeeVariant::ConBeeII => "ConBee II USB Dongle",
            ConBeeVariant::RaspBee => "RaspBee Module",
            ConBeeVariant::RaspBeeII => "RaspBee II Module",
        }
    }
    
    fn device_id(&self) -> String {
        format!("conbee:{}", self.port_name)
    }
    
    async fn shutdown(&mut self) -> HalResult<()> {
        self.port = None;
        self.active = false;
        Ok(())
    }
    
    fn is_active(&self) -> bool {
        self.active
    }
}

struct Frame {
    cmd: u8,
    seq: u8,
    status: u8,
    data: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConBeeVariant {
    ConBee,
    ConBeeII,
    RaspBee,
    RaspBeeII,
}

impl Drop for ConBee {
    fn drop(&mut self) {
        self.active = false;
    }
}