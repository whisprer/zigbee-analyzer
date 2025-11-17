use zigbee_core::packet::RawPacket;
use zigbee_hal::{
    traits::{ZigbeeCapture, PacketInjection},
    capabilities::DeviceCapabilities,
    error::{HalError, HalResult},
};
use async_trait::async_trait;
use serialport::{SerialPort, SerialPortType};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use std::io::{Read, Write};

/// TI CC2652 USB Zigbee Sniffer driver
/// Supports both CC2652R and CC2652P variants
#[allow(dead_code)]
pub struct CC2652 {
    port: Option<Arc<Mutex<Box<dyn SerialPort>>>>,
    port_name: String,
    channel: u8,
    active: bool,
    capabilities: DeviceCapabilities,
    buffer: Vec<u8>,
    supports_injection: bool,
}

// CC2652 protocol constants (uses similar protocol to CC2531 but extended)
const SOF: u8 = 0xFE;                    // Start of Frame
const CMD_PING: u8 = 0x01;               // Ping command
const CMD_START: u8 = 0x82;              // Start capture
const CMD_STOP: u8 = 0x83;               // Stop capture  
const CMD_SET_CHANNEL: u8 = 0x84;        // Set channel
const CMD_PACKET: u8 = 0x87;             // Captured packet indicator
const CMD_SET_POWER: u8 = 0x85;          // Set TX power (if supported)
const CMD_INJECT: u8 = 0x88;             // Inject packet (if supported)
const CMD_GET_CONFIG: u8 = 0x89;         // Get device configuration

// USB VID/PID for various CC2652 variants
const TI_VID: u16 = 0x0451;
const CC2652_PID: u16 = 0x16c8;          // Standard CC2652
const CC2652P_PID: u16 = 0x16c9;         // CC2652P (high power variant)
const CC2652RB_PID: u16 = 0x16ca;        // CC2652RB variant

impl CC2652 {
    /// Create a new CC2652 driver instance
    pub fn new() -> HalResult<Self> {
        let (port_name, variant) = Self::find_device()?;
        let supports_injection = variant == CC2652Variant::CC2652P;
        
        let mut capabilities = DeviceCapabilities::basic_capture();
        
        // CC2652 has better specs than CC2531
        capabilities.max_sample_rate = 500;
        capabilities.buffer_size = 512;
        capabilities.hardware_timestamps = true;
        capabilities.energy_detection = true;
        
        // P variant supports TX
        if supports_injection {
            capabilities.packet_injection = true;
            capabilities.tx_power_range = Some((-20, 5));
        }
        
        Ok(Self {
            port: None,
            port_name,
            channel: 11,
            active: false,
            capabilities,
            buffer: Vec::with_capacity(512),
            supports_injection,
        })
    }
    
    /// Find CC2652 device on system
    fn find_device() -> HalResult<(String, CC2652Variant)> {
        let ports = serialport::available_ports()
            .map_err(|e| HalError::HardwareError(format!("Failed to enumerate ports: {}", e)))?;
        
        for port in ports {
            if let SerialPortType::UsbPort(info) = &port.port_type {
                if info.vid == TI_VID {
                    let variant = match info.pid {
                        CC2652_PID => Some(CC2652Variant::CC2652),
                        CC2652P_PID => Some(CC2652Variant::CC2652P),
                        CC2652RB_PID => Some(CC2652Variant::CC2652RB),
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
    pub fn new_with_port(port_name: String) -> HalResult<Self> {
        let mut capabilities = DeviceCapabilities::basic_capture();
        capabilities.max_sample_rate = 500;
        capabilities.buffer_size = 512;
        capabilities.hardware_timestamps = true;
        capabilities.energy_detection = true;
        
        Ok(Self {
            port: None,
            port_name,
            channel: 11,
            active: false,
            capabilities,
            buffer: Vec::with_capacity(512),
            supports_injection: false, // Will be detected on init
        })
    }
    
    /// Open serial port connection
    fn open_port(&mut self) -> HalResult<()> {
        let port = serialport::new(&self.port_name, 115200)
            .timeout(Duration::from_millis(100))
            .data_bits(serialport::DataBits::Eight)
            .stop_bits(serialport::StopBits::One)
            .parity(serialport::Parity::None)
            .flow_control(serialport::FlowControl::None)
            .open()
            .map_err(|e| HalError::SerialError(format!("Failed to open port: {}", e)))?;
        
        self.port = Some(Arc::new(Mutex::new(port)));
        Ok(())
    }
    
    /// Send a command to the CC2652
    fn send_command(&mut self, cmd: u8, data: &[u8]) -> HalResult<()> {
        let port = self.port.as_ref()
            .ok_or(HalError::NotInitialized)?;
        
        let mut port_guard = port.lock()
            .map_err(|e| HalError::SerialError(format!("Lock failed: {}", e)))?;
        
        let len = data.len() as u8;
        
        // Build frame: SOF | LEN | CMD | DATA | FCS
        let mut frame = vec![SOF, len, cmd];
        frame.extend_from_slice(data);
        
        // Calculate FCS (XOR of all bytes except SOF)
        let fcs = frame[1..].iter().fold(0u8, |acc, &b| acc ^ b);
        frame.push(fcs);
        
        port_guard.write_all(&frame)
            .map_err(|e| HalError::SerialError(format!("Write failed: {}", e)))?;
        
        port_guard.flush()
            .map_err(|e| HalError::SerialError(format!("Flush failed: {}", e)))?;
        
        Ok(())
    }
    
    /// Read and parse a frame from CC2652
    fn read_frame(&mut self, timeout: Duration) -> HalResult<Option<Frame>> {
        let port = self.port.as_ref()
            .ok_or(HalError::NotInitialized)?;
        
        let mut port_guard = port.lock()
            .map_err(|e| HalError::SerialError(format!("Lock failed: {}", e)))?;
        
        // Set timeout for this read
        port_guard.set_timeout(timeout)
            .map_err(|e| HalError::SerialError(format!("Set timeout failed: {}", e)))?;
        
        // Look for SOF byte
        let mut sof_buf = [0u8; 1];
        loop {
            match port_guard.read(&mut sof_buf) {
                Ok(1) => {
                    if sof_buf[0] == SOF {
                        break;
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
        }
        
        // Read length and command
        let mut header = [0u8; 2];
        port_guard.read_exact(&mut header)
            .map_err(|e| HalError::SerialError(format!("Read header failed: {}", e)))?;
        
        let len = header[0] as usize;
        let cmd = header[1];
        
        // Read data
        let mut data = vec![0u8; len];
        if len > 0 {
            port_guard.read_exact(&mut data)
                .map_err(|e| HalError::SerialError(format!("Read data failed: {}", e)))?;
        }
        
        // Read FCS
        let mut fcs_buf = [0u8; 1];
        port_guard.read_exact(&mut fcs_buf)
            .map_err(|e| HalError::SerialError(format!("Read FCS failed: {}", e)))?;
        
        // Verify FCS
        let calculated_fcs = [header[0], header[1]]
            .iter()
            .chain(data.iter())
            .fold(0u8, |acc, &b| acc ^ b);
        
        if calculated_fcs != fcs_buf[0] {
            return Err(HalError::InvalidPacket("FCS mismatch".to_string()));
        }
        
        Ok(Some(Frame { cmd, data }))
    }
    
    /// Parse a captured packet from CC2652 data
    fn parse_packet(&self, data: &[u8]) -> HalResult<RawPacket> {
        if data.len() < 8 {
            return Err(HalError::InvalidPacket(format!(
                "Packet too short: {} bytes",
                data.len()
            )));
        }
        
        // CC2652 packet format (similar to CC2531 but with enhancements):
        // Bytes 0-3: Timestamp (little-endian, microseconds)
        // Byte 4: Packet length
        // Bytes 5..(5+len): Packet data
        // Byte (5+len): RSSI (signed)
        // Byte (5+len+1): LQI
        // Byte (5+len+2): FCS OK flag
        
        let _timestamp_us = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let pkt_len = data[4] as usize;
        
        if data.len() < 5 + pkt_len + 3 {
            return Err(HalError::InvalidPacket("Incomplete packet data".to_string()));
        }
        
        let packet_data = &data[5..5 + pkt_len];
        let rssi = data[5 + pkt_len] as i8;
        let lqi = data[5 + pkt_len + 1];
        let fcs_ok = data[5 + pkt_len + 2] != 0;
        
        if !fcs_ok {
            // Still return packet but user can validate
            // Some applications want to see bad packets
        }
        
        Ok(RawPacket {
            timestamp: SystemTime::now(),
            channel: self.channel,
            rssi,
            lqi,
            data: packet_data.to_vec(),
        })
    }
    
    /// Query device configuration
    async fn query_config(&mut self) -> HalResult<()> {
        self.send_command(CMD_GET_CONFIG, &[])?;
        
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        match self.read_frame(Duration::from_millis(500))? {
            Some(frame) => {
                // Parse config response to detect features
                if frame.data.len() >= 1 {
                    let features = frame.data[0];
                    self.supports_injection = (features & 0x01) != 0;
                    
                    if self.supports_injection {
                        self.capabilities.packet_injection = true;
                        self.capabilities.tx_power_range = Some((-20, 5));
                    }
                }
                Ok(())
            }
            None => {
                // Older firmware may not support config query
                Ok(())
            }
        }
    }
}

#[async_trait]
impl ZigbeeCapture for CC2652 {
    async fn initialize(&mut self) -> HalResult<()> {
        // Open serial port
        self.open_port()?;
        
        // Send ping to verify device is responding
        self.send_command(CMD_PING, &[])?;
        
        // Wait for response
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Read response (should get a ping response)
        match self.read_frame(Duration::from_millis(500))? {
            Some(_frame) => {
                // Device responded, we're good
            }
            None => {
                return Err(HalError::HardwareError(
                    "No response from device".to_string()
                ));
            }
        }
        
        // Query device configuration to detect features
        let _ = self.query_config().await;
        
        // Start capture on current channel
        self.send_command(CMD_START, &[])?;
        
        self.active = true;
        Ok(())
    }
    
    async fn set_channel(&mut self, channel: u8) -> HalResult<()> {
        if !(11..=26).contains(&channel) {
            return Err(HalError::InvalidChannel(channel));
        }
        
        // Stop capture
        if self.active {
            self.send_command(CMD_STOP, &[])?;
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        
        // Set new channel
        self.send_command(CMD_SET_CHANNEL, &[channel])?;
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        // Restart capture
        if self.active {
            self.send_command(CMD_START, &[])?;
        }
        
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
                    if frame.cmd == CMD_PACKET {
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
                if frame.cmd == CMD_PACKET {
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
        "TI CC2652 USB Dongle"
    }
    
    fn device_id(&self) -> String {
        format!("cc2652:{}", self.port_name)
    }
    
    async fn shutdown(&mut self) -> HalResult<()> {
        if let Some(_port) = self.port.as_mut() {
            // Send stop command
            let _ = self.send_command(CMD_STOP, &[]);
        }
        
        self.port = None;
        self.active = false;
        Ok(())
    }
    
    fn is_active(&self) -> bool {
        self.active
    }
}

#[async_trait]
impl PacketInjection for CC2652 {
    async fn inject_packet(&mut self, packet: &RawPacket) -> HalResult<()> {
        if !self.supports_injection {
            return Err(HalError::NotSupported);
        }
        
        if !self.active {
            return Err(HalError::NotInitialized);
        }
        
        // Build injection payload: channel | data_len | data
        let mut payload = vec![packet.channel, packet.data.len() as u8];
        payload.extend_from_slice(&packet.data);
        
        self.send_command(CMD_INJECT, &payload)?;
        
        // Wait for transmission to complete
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        Ok(())
    }
    
    async fn set_tx_power(&mut self, dbm: i8) -> HalResult<()> {
        if !self.supports_injection {
            return Err(HalError::NotSupported);
        }
        
        let (min, max) = self.capabilities.tx_power_range
            .ok_or(HalError::NotSupported)?;
        
        if dbm < min || dbm > max {
            return Err(HalError::ConfigError(
                format!("TX power {} dBm out of range ({} to {})", dbm, min, max)
            ));
        }
        
        // CC2652 uses signed byte for power
        self.send_command(CMD_SET_POWER, &[dbm as u8])?;
        
        Ok(())
    }
    
    fn get_tx_power(&self) -> HalResult<i8> {
        // Default/current power - would need to track this
        Ok(0)
    }
    
    fn tx_power_range(&self) -> (i8, i8) {
        self.capabilities.tx_power_range.unwrap_or((-20, 5))
    }
}

/// Internal frame structure
struct Frame {
    cmd: u8,
    data: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CC2652Variant {
    CC2652,      // Standard variant
    CC2652P,     // High power variant with PA
    CC2652RB,    // RB variant
}

impl Drop for CC2652 {
    fn drop(&mut self) {
        // Ensure we stop capture on drop
        if self.port.is_some() {
            let _ = self.send_command(CMD_STOP, &[]);
        }
    }
}