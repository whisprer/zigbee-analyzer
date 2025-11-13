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
const CMD_APS_DATA_INDICATION: u8 = 0x17;  // Captured packet

// Parameter IDs
const PARAM_MAC_ADDRESS: u8 = 0x01;
const PARAM_NETWORK_PANID: u8 = 0x05;
const PARAM_NETWORK_ADDR: u8 = 0x07;
const PARAM_CHANNEL_MASK: u8 = 0x0A;
const PARAM_APS_EXTENDED_PANID: u8 = 0x0B;
const PARAM_TRUST_CENTER_ADDRESS: u8 = 0x0E;
const PARAM_NETWORK_KEY: u8 = 0x18;

// ConBee device states
const STATE_NETWORK_LOST: u8 = 0x00;
const STATE_JOINING: u8 = 0x01;
const STATE_CONNECTED: u8 = 0x02;
const STATE_LEAVING: u8 = 0x03;

// USB VID/PID for ConBee devices
const DRESDEN_VID: u16 = 0x1cf1;
const CONBEE_PID: u16 = 0x0030;      // ConBee I
const CONBEE_II_PID: u16 = 0x0031;   // ConBee II
const RASPBEE_PID: u16 = 0x0028;     // RaspBee
const RASPBEE_II_PID: u16 = 0x0029;  // RaspBee II

impl ConBee {
    /// Create a new ConBee driver instance
    pub fn new() -> HalResult<Self> {
        let (port_name, variant) = Self::find_device()?;
        
        let mut capabilities = DeviceCapabilities::basic_capture();
        
        // ConBee has good specs
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
            buffer: Vec::with_capacity(1024),
            variant,
            firmware_version: None,
        })
    }
    
    /// Find ConBee device on system
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
    
    /// Create driver for specific port (used by registry)
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
            buffer: Vec::with_capacity(1024),
            variant,
            firmware_version: None,
        })
    }
    
    /// Open serial port connection
    fn open_port(&mut self) -> HalResult<()> {
        // ConBee uses 38400 baud for ConBee I, 115200 for ConBee II
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
    
    /// Encode data with SLIP framing
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
    
    /// Decode SLIP framed data
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
                    SLIP_END => {
                        // Frame delimiter, ignore
                    }
                    SLIP_ESC => {
                        escape_next = true;
                    }
                    _ => {
                        decoded.push(byte);
                    }
                }
            }
        }
        
        decoded
    }
    
    /// Send a command to the ConBee
    fn send_command(&mut self, cmd: u8, payload: &[u8]) -> HalResult<()> {
        let port = self.port.as_mut()
            .ok_or(HalError::NotInitialized)?;
        
        // Build frame: CMD | SEQ | 0x00 | LEN_L | LEN_H | PAYLOAD
        let seq = 0u8; // Sequence number (we'll use 0 for simplicity)
        let len = payload.len() as u16;
        
        let mut frame = vec![cmd, seq, 0x00, (len & 0xFF) as u8, ((len >> 8) & 0xFF) as u8];
        frame.extend_from_slice(payload);
        
        // SLIP encode
        let encoded = self.slip_encode(&frame);
        
        port.write_all(&encoded)
            .map_err(|e| HalError::SerialError(format!("Write failed: {}", e)))?;
        
        port.flush()
            .map_err(|e| HalError::SerialError(format!("Flush failed: {}", e)))?;
        
        Ok(())
    }
    
    /// Read a SLIP frame from ConBee
    fn read_frame(&mut self, timeout: Duration) -> HalResult<Option<Frame>> {
        let port = self.port.as_mut()
            .ok_or(HalError::NotInitialized)?;
        
        port.set_timeout(timeout)
            .map_err(|e| HalError::SerialError(format!("Set timeout failed: {}", e)))?;
        
        let mut raw_buffer = Vec::new();
        let mut byte_buf = [0u8; 1];
        let mut in_frame = false;
        
        // Read until we get a complete SLIP frame
        loop {
            match port.read(&mut byte_buf) {
                Ok(1) => {
                    let byte = byte_buf[0];
                    
                    if byte == SLIP_END {
                        if in_frame && !raw_buffer.is_empty() {
                            // End of frame
                            break;
                        } else {
                            // Start of frame
                            in_frame = true;
                            raw_buffer.clear();
                        }
                    } else if in_frame {
                        raw_buffer.push(byte);
                    }
                }
                Ok(_) => continue,
                Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {
                    return Ok(None);
                }
                Err(e) => {
                    return Err(HalError::SerialError(format!("Read failed: {}", e)));
                }
            }
            
            // Safety: don't accumulate too much
            if raw_buffer.len() > 2048 {
                return Err(HalError::InvalidPacket("Frame too large".to_string()));
            }
        }
        
        // Decode SLIP
        let decoded = self.slip_decode(&raw_buffer);
        
        if decoded.len() < 5 {
            return Err(HalError::InvalidPacket("Frame too short".to_string()));
        }
        
        let cmd = decoded[0];
        let _seq = decoded[1];
        let _status = decoded[2];
        let len = decoded[3] as usize | ((decoded[4] as usize) << 8);
        
        if decoded.len() < 5 + len {
            return Err(HalError::InvalidPacket("Incomplete frame data".to_string()));
        }
        
        let data = decoded[5..5 + len].to_vec();
        
        Ok(Some(Frame { cmd, data }))
    }
    
    /// Query firmware version
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
    
    /// Set channel by writing channel mask parameter
    async fn set_channel_mask(&mut self, channel: u8) -> HalResult<()> {
        // Channel mask is a 32-bit value where each bit represents a channel
        // Bit 11 = channel 11, bit 12 = channel 12, etc.
        let channel_mask: u32 = 1 << channel;
        
        let mut payload = vec![PARAM_CHANNEL_MASK];
        payload.extend_from_slice(&channel_mask.to_le_bytes());
        
        self.send_command(CMD_WRITE_PARAMETER, &payload)?;
        
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        Ok(())
    }
    
    /// Parse a captured packet from ConBee data
    fn parse_packet(&self, data: &[u8]) -> HalResult<RawPacket> {
        // ConBee APS data indication format is complex
        // For simplicity, we'll extract the relevant fields
        
        if data.len() < 20 {
            return Err(HalError::InvalidPacket(format!(
                "Packet too short: {} bytes",
                data.len()
            )));
        }
        
        // ConBee includes full APS/NWK/MAC data
        // The actual 802.15.4 frame starts at an offset
        // This is a simplified parser - real implementation would be more complex
        
        // Typical offsets (these vary by firmware version):
        // Bytes 0-1: Dest addr
        // Bytes 2-3: Profile ID
        // Bytes 4-5: Cluster ID
        // Byte 6: Dest endpoint
        // Byte 7: Src endpoint
        // ...
        // The MAC frame data is further in
        
        // For now, we'll extract what we can
        let rssi = if data.len() > 10 { data[10] as i8 } else { -50 };
        let lqi = if data.len() > 11 { data[11] } else { 200 };
        
        // Try to find the actual MAC frame in the payload
        // ConBee wraps packets heavily, so we need to unwrap
        let mac_frame_start = 12; // Approximate offset
        
        if data.len() <= mac_frame_start {
            return Err(HalError::InvalidPacket("No MAC frame data".to_string()));
        }
        
        let mac_data = data[mac_frame_start..].to_vec();
        
        Ok(RawPacket {
            timestamp: SystemTime::now(),
            channel: self.channel,
            rssi,
            lqi,
            data: mac_data,
        })
    }
}

#[async_trait]
impl ZigbeeCapture for ConBee {
    async fn initialize(&mut self) -> HalResult<()> {
        // Open serial port
        self.open_port()?;
        
        // Small delay for device to settle
        tokio::time::sleep(Duration::from_millis(200)).await;
        
        // Query firmware version
        self.query_version().await?;
        
        // Set initial channel
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
        
        // Keep reading frames until we get a packet
        loop {
            match self.read_frame(Duration::from_secs(5))? {
                Some(frame) => {
                    if frame.cmd == CMD_APS_DATA_INDICATION {
                        return self.parse_packet(&frame.data);
                    }
                    // Ignore other frame types, continue reading
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
        
        // Non-blocking read with short timeout
        match self.read_frame(Duration::from_millis(1))? {
            Some(frame) => {
                if frame.cmd == CMD_APS_DATA_INDICATION {
                    Ok(Some(self.parse_packet(&frame.data)?))
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

/// Internal frame structure
struct Frame {
    cmd: u8,
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
        // Clean shutdown
        self.active = false;
    }
}