use zigbee_core::packet::RawPacket;
use zigbee_hal::{
    traits::ZigbeeCapture,
    capabilities::DeviceCapabilities,
    error::{HalError, HalResult},
};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// PCAP file reader for analyzing recorded Zigbee captures
pub struct PcapReader {
    file_path: PathBuf,
    packets: Vec<StoredPacket>,
    current_index: usize,
    channel: u8,
    active: bool,
    capabilities: DeviceCapabilities,
    playback_speed: f32,
    loop_playback: bool,
}

#[derive(Debug, Clone)]
struct StoredPacket {
    timestamp: SystemTime,
    channel: u8,
    rssi: i8,
    lqi: u8,
    data: Vec<u8>,
}

// PCAP file format constants
const PCAP_MAGIC_NUMBER: u32 = 0xa1b2c3d4;
const PCAP_MAGIC_NUMBER_NS: u32 = 0xa1b23c4d;  // Nanosecond precision
const LINKTYPE_IEEE802_15_4: u16 = 195;         // DLT for 802.15.4
const LINKTYPE_IEEE802_15_4_WITHFCS: u16 = 230; // 802.15.4 with FCS
const LINKTYPE_IEEE802_15_4_NOFCS: u16 = 231;   // 802.15.4 without FCS

impl PcapReader {
    /// Create a new PCAP reader from file
    pub fn new<P: AsRef<Path>>(file_path: P) -> HalResult<Self> {
        let file_path = file_path.as_ref().to_path_buf();
        
        if !file_path.exists() {
            return Err(HalError::HardwareError(format!(
                "PCAP file not found: {:?}",
                file_path
            )));
        }
        
        let mut capabilities = DeviceCapabilities::basic_capture();
        capabilities.hardware_timestamps = true;
        
        Ok(Self {
            file_path,
            packets: Vec::new(),
            current_index: 0,
            channel: 11,
            active: false,
            capabilities,
            playback_speed: 1.0,
            loop_playback: false,
        })
    }
    
    /// Set playback speed multiplier (1.0 = realtime, 0.0 = as fast as possible)
    pub fn set_playback_speed(&mut self, speed: f32) {
        self.playback_speed = speed.max(0.0);
    }
    
    /// Enable or disable looping playback
    pub fn set_loop(&mut self, enabled: bool) {
        self.loop_playback = enabled;
    }
    
    /// Get total number of packets in file
    pub fn packet_count(&self) -> usize {
        self.packets.len()
    }
    
    /// Get current playback position
    pub fn current_position(&self) -> usize {
        self.current_index
    }
    
    /// Seek to specific packet index
    pub fn seek(&mut self, index: usize) {
        self.current_index = index.min(self.packets.len());
    }
    
    /// Reset to beginning
    pub fn reset(&mut self) {
        self.current_index = 0;
    }
    
    /// Load and parse the PCAP file
    fn load_pcap(&mut self) -> HalResult<()> {
        use std::io::Read;
        
        let mut file = File::open(&self.file_path)
            .map_err(|e| HalError::IoError(e))?;
        
        // Read global header (24 bytes)
        let mut header = [0u8; 24];
        file.read_exact(&mut header)
            .map_err(|e| HalError::IoError(e))?;
        
        let magic = u32::from_le_bytes([header[0], header[1], header[2], header[3]]);
        let _version_major = u16::from_le_bytes([header[4], header[5]]);
        let _version_minor = u16::from_le_bytes([header[6], header[7]]);
        let _thiszone = i32::from_le_bytes([header[8], header[9], header[10], header[11]]);
        let _sigfigs = u32::from_le_bytes([header[12], header[13], header[14], header[15]]);
        let _snaplen = u32::from_le_bytes([header[16], header[17], header[18], header[19]]);
        let linktype = u32::from_le_bytes([header[20], header[21], header[22], header[23]]);
        
        // Verify magic number
        let nanosecond_precision = match magic {
            PCAP_MAGIC_NUMBER => false,
            PCAP_MAGIC_NUMBER_NS => true,
            _ => {
                return Err(HalError::InvalidPacket(format!(
                    "Invalid PCAP magic number: 0x{:08x}",
                    magic
                )));
            }
        };
        
        // Verify link type is IEEE 802.15.4
        let linktype_u16 = linktype as u16;
        if linktype_u16 != LINKTYPE_IEEE802_15_4 
            && linktype_u16 != LINKTYPE_IEEE802_15_4_WITHFCS
            && linktype_u16 != LINKTYPE_IEEE802_15_4_NOFCS {
            return Err(HalError::InvalidPacket(format!(
                "Unsupported link type: {} (expected IEEE 802.15.4)",
                linktype
            )));
        }
        
        // Read packet records
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .map_err(|e| HalError::IoError(e))?;
        
        let mut offset = 0;
        while offset + 16 <= buffer.len() {
            // Read packet header (16 bytes)
            let ts_sec = u32::from_le_bytes([
                buffer[offset],
                buffer[offset + 1],
                buffer[offset + 2],
                buffer[offset + 3],
            ]);
            let ts_usec_or_nsec = u32::from_le_bytes([
                buffer[offset + 4],
                buffer[offset + 5],
                buffer[offset + 6],
                buffer[offset + 7],
            ]);
            let incl_len = u32::from_le_bytes([
                buffer[offset + 8],
                buffer[offset + 9],
                buffer[offset + 10],
                buffer[offset + 11],
            ]) as usize;
            let _orig_len = u32::from_le_bytes([
                buffer[offset + 12],
                buffer[offset + 13],
                buffer[offset + 14],
                buffer[offset + 15],
            ]);
            
            offset += 16;
            
            if offset + incl_len > buffer.len() {
                break; // Incomplete packet
            }
            
            // Extract packet data
            let packet_data = buffer[offset..offset + incl_len].to_vec();
            offset += incl_len;
            
            // Convert timestamp
            let timestamp = if nanosecond_precision {
                UNIX_EPOCH + Duration::from_secs(ts_sec as u64) 
                    + Duration::from_nanos(ts_usec_or_nsec as u64)
            } else {
                UNIX_EPOCH + Duration::from_secs(ts_sec as u64) 
                    + Duration::from_micros(ts_usec_or_nsec as u64)
            };
            
            // Try to extract metadata (channel, RSSI, LQI) if present
            // Some PCAP formats include this as pseudo-headers
            let (channel, rssi, lqi, data) = self.parse_packet_metadata(&packet_data);
            
            self.packets.push(StoredPacket {
                timestamp,
                channel,
                rssi,
                lqi,
                data,
            });
        }
        
        Ok(())
    }
    
    /// Parse packet metadata if present (some sniffers add custom headers)
    fn parse_packet_metadata(&self, data: &[u8]) -> (u8, i8, u8, Vec<u8>) {
        // Check for common Zigbee sniffer metadata formats
        
        // Format 1: TI CC2531 PCAP format (has metadata prepended)
        if data.len() > 2 && data[0] == 0x00 && data[1] <= 26 {
            // Might be TI format with channel in byte 1
            let channel = data[1];
            if (11..=26).contains(&channel) && data.len() > 4 {
                let rssi = data[2] as i8;
                let lqi = data[3];
                return (channel, rssi, lqi, data[4..].to_vec());
            }
        }
        
        // Format 2: Check for FCS Radio Tap header (used by some tools)
        // This is more complex - simplified version here
        
        // Default: assume no metadata, use defaults
        (self.channel, -50, 200, data.to_vec())
    }
}

#[async_trait]
impl ZigbeeCapture for PcapReader {
    async fn initialize(&mut self) -> HalResult<()> {
        // Load the PCAP file
        self.load_pcap()?;
        
        if self.packets.is_empty() {
            return Err(HalError::HardwareError(
                "PCAP file contains no packets".to_string()
            ));
        }
        
        self.active = true;
        self.current_index = 0;
        
        Ok(())
    }
    
    async fn set_channel(&mut self, channel: u8) -> HalResult<()> {
        if !(11..=26).contains(&channel) {
            return Err(HalError::InvalidChannel(channel));
        }
        
        // For PCAP files, this is just informational
        // We can't actually change the channel of recorded data
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
        
        if self.current_index >= self.packets.len() {
            if self.loop_playback {
                self.current_index = 0;
            } else {
                return Err(HalError::HardwareError("End of file".to_string()));
            }
        }
        
        let stored = &self.packets[self.current_index];
        
        // Simulate timing if playback_speed > 0
        if self.playback_speed > 0.0 && self.current_index > 0 {
            let prev = &self.packets[self.current_index - 1];
            
            if let Ok(duration) = stored.timestamp.duration_since(prev.timestamp) {
                let delay = duration.mul_f32(self.playback_speed);
                tokio::time::sleep(delay).await;
            }
        }
        
        let packet = RawPacket {
            timestamp: stored.timestamp,
            channel: stored.channel,
            rssi: stored.rssi,
            lqi: stored.lqi,
            data: stored.data.clone(),
        };
        
        self.current_index += 1;
        
        Ok(packet)
    }
    
    fn try_capture_packet(&mut self) -> HalResult<Option<RawPacket>> {
        if !self.active {
            return Err(HalError::NotInitialized);
        }
        
        if self.current_index >= self.packets.len() {
            if self.loop_playback {
                self.current_index = 0;
            } else {
                return Ok(None);
            }
        }
        
        let stored = &self.packets[self.current_index];
        
        let packet = RawPacket {
            timestamp: stored.timestamp,
            channel: stored.channel,
            rssi: stored.rssi,
            lqi: stored.lqi,
            data: stored.data.clone(),
        };
        
        self.current_index += 1;
        
        Ok(Some(packet))
    }
    
    fn capabilities(&self) -> &DeviceCapabilities {
        &self.capabilities
    }
    
    fn device_name(&self) -> &str {
        "PCAP File Reader"
    }
    
    fn device_id(&self) -> String {
        format!("pcap:{}", self.file_path.display())
    }
    
    async fn shutdown(&mut self) -> HalResult<()> {
        self.active = false;
        Ok(())
    }
    
    fn is_active(&self) -> bool {
        self.active
    }
}

/// PCAP file writer for recording captures
pub struct PcapWriter {
    file: Option<File>,
    file_path: PathBuf,
    nanosecond_precision: bool,
}

impl PcapWriter {
    /// Create a new PCAP writer
    pub fn new<P: AsRef<Path>>(file_path: P, nanosecond_precision: bool) -> HalResult<Self> {
        Ok(Self {
            file: None,
            file_path: file_path.as_ref().to_path_buf(),
            nanosecond_precision,
        })
    }
    
    /// Open the file and write PCAP header
    pub fn open(&mut self) -> HalResult<()> {
        use std::io::Write;
        
        let mut file = File::create(&self.file_path)
            .map_err(|e| HalError::IoError(e))?;
        
        // Write global header
        let magic = if self.nanosecond_precision {
            PCAP_MAGIC_NUMBER_NS
        } else {
            PCAP_MAGIC_NUMBER
        };
        
        let header = [
            &magic.to_le_bytes()[..],              // Magic number
            &2u16.to_le_bytes()[..],               // Version major
            &4u16.to_le_bytes()[..],               // Version minor
            &0i32.to_le_bytes()[..],               // Timezone (GMT)
            &0u32.to_le_bytes()[..],               // Timestamp accuracy
            &65535u32.to_le_bytes()[..],           // Snaplen
            &(LINKTYPE_IEEE802_15_4_WITHFCS as u32).to_le_bytes()[..], // Link type
        ].concat();
        
        file.write_all(&header)
            .map_err(|e| HalError::IoError(e))?;
        
        self.file = Some(file);
        Ok(())
    }
    
    /// Write a packet to the PCAP file
    pub fn write_packet(&mut self, packet: &RawPacket) -> HalResult<()> {
        use std::io::Write;
        
        let file = self.file.as_mut()
            .ok_or(HalError::NotInitialized)?;
        
        // Convert timestamp
        let duration = packet.timestamp.duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0));
        
        let ts_sec = duration.as_secs() as u32;
        let ts_frac = if self.nanosecond_precision {
            duration.subsec_nanos()
        } else {
            duration.subsec_micros()
        };
        
        let packet_len = packet.data.len() as u32;
        
        // Write packet header
        let pkt_header = [
            &ts_sec.to_le_bytes()[..],
            &ts_frac.to_le_bytes()[..],
            &packet_len.to_le_bytes()[..],
            &packet_len.to_le_bytes()[..],
        ].concat();
        
        file.write_all(&pkt_header)
            .map_err(|e| HalError::IoError(e))?;
        
        // Write packet data
        file.write_all(&packet.data)
            .map_err(|e| HalError::IoError(e))?;
        
        file.flush()
            .map_err(|e| HalError::IoError(e))?;
        
        Ok(())
    }
    
    /// Close the file
    pub fn close(&mut self) -> HalResult<()> {
        if let Some(mut file) = self.file.take() {
            use std::io::Write;
            file.flush()
                .map_err(|e| HalError::IoError(e))?;
        }
        Ok(())
    }
}

impl Drop for PcapWriter {
    fn drop(&mut self) {
        let _ = self.close();
    }
}