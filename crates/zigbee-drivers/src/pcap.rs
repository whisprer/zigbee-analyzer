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

const PCAP_MAGIC_NUMBER: u32 = 0xa1b2c3d4;
const PCAP_MAGIC_NUMBER_NS: u32 = 0xa1b23c4d;
const PCAP_MAGIC_SWAPPED: u32 = 0xd4c3b2a1;
const PCAP_MAGIC_NS_SWAPPED: u32 = 0x4d3cb2a1;

const LINKTYPE_IEEE802_15_4: u16 = 195;
const LINKTYPE_IEEE802_15_4_WITHFCS: u16 = 230;
const LINKTYPE_IEEE802_15_4_NOFCS: u16 = 231;

impl PcapReader {
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
    
    pub fn set_playback_speed(&mut self, speed: f32) {
        self.playback_speed = speed.max(0.0);
    }
    
    pub fn set_loop(&mut self, enabled: bool) {
        self.loop_playback = enabled;
    }
    
    pub fn packet_count(&self) -> usize {
        self.packets.len()
    }
    
    pub fn current_position(&self) -> usize {
        self.current_index
    }
    
    pub fn seek(&mut self, index: usize) {
        self.current_index = index.min(self.packets.len());
    }
    
    pub fn reset(&mut self) {
        self.current_index = 0;
    }
    
    fn load_pcap(&mut self) -> HalResult<()> {
        use std::io::Read;
        
        let mut file = File::open(&self.file_path)
            .map_err(|e| HalError::IoError(e))?;
        
        let mut header = [0u8; 24];
        file.read_exact(&mut header)
            .map_err(|e| HalError::IoError(e))?;
        
        let magic = u32::from_le_bytes([header[0], header[1], header[2], header[3]]);
        
        let (nanosecond_precision, swapped) = match magic {
            PCAP_MAGIC_NUMBER => (false, false),
            PCAP_MAGIC_NUMBER_NS => (true, false),
            PCAP_MAGIC_SWAPPED => (false, true),
            PCAP_MAGIC_NS_SWAPPED => (true, true),
            _ => {
                return Err(HalError::InvalidPacket(format!(
                    "Invalid PCAP magic number: 0x{:08x}",
                    magic
                )));
            }
        };
        
        let read_u32 = |bytes: [u8; 4]| -> u32 {
            if swapped {
                u32::from_be_bytes(bytes)
            } else {
                u32::from_le_bytes(bytes)
            }
        };
        
        let linktype = read_u32([header[20], header[21], header[22], header[23]]);
        
        let linktype_u16 = linktype as u16;
        if linktype_u16 != LINKTYPE_IEEE802_15_4 
            && linktype_u16 != LINKTYPE_IEEE802_15_4_WITHFCS
            && linktype_u16 != LINKTYPE_IEEE802_15_4_NOFCS {
            return Err(HalError::InvalidPacket(format!(
                "Unsupported link type: {} (expected IEEE 802.15.4)",
                linktype
            )));
        }
        
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .map_err(|e| HalError::IoError(e))?;
        
        let mut offset = 0;
        while offset + 16 <= buffer.len() {
            let ts_sec = read_u32([
                buffer[offset],
                buffer[offset + 1],
                buffer[offset + 2],
                buffer[offset + 3],
            ]);
            let ts_usec_or_nsec = read_u32([
                buffer[offset + 4],
                buffer[offset + 5],
                buffer[offset + 6],
                buffer[offset + 7],
            ]);
            let incl_len = read_u32([
                buffer[offset + 8],
                buffer[offset + 9],
                buffer[offset + 10],
                buffer[offset + 11],
            ]) as usize;
            
            offset += 16;
            
            if offset + incl_len > buffer.len() {
                break;
            }
            
            let packet_data = buffer[offset..offset + incl_len].to_vec();
            offset += incl_len;
            
            let timestamp = if nanosecond_precision {
                UNIX_EPOCH + Duration::from_secs(ts_sec as u64) 
                    + Duration::from_nanos(ts_usec_or_nsec as u64)
            } else {
                UNIX_EPOCH + Duration::from_secs(ts_sec as u64) 
                    + Duration::from_micros(ts_usec_or_nsec as u64)
            };
            
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
    
    fn parse_packet_metadata(&self, data: &[u8]) -> (u8, i8, u8, Vec<u8>) {
        if data.len() > 2 && data[0] == 0x00 && data[1] <= 26 {
            let channel = data[1];
            if (11..=26).contains(&channel) && data.len() > 4 {
                let rssi = data[2] as i8;
                let lqi = data[3];
                return (channel, rssi, lqi, data[4..].to_vec());
            }
        }
        
        (self.channel, -50, 200, data.to_vec())
    }
}

#[async_trait]
impl ZigbeeCapture for PcapReader {
    async fn initialize(&mut self) -> HalResult<()> {
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

pub struct PcapWriter {
    file: Option<File>,
    file_path: PathBuf,
    nanosecond_precision: bool,
}

impl PcapWriter {
    pub fn new<P: AsRef<Path>>(file_path: P, nanosecond_precision: bool) -> HalResult<Self> {
        Ok(Self {
            file: None,
            file_path: file_path.as_ref().to_path_buf(),
            nanosecond_precision,
        })
    }
    
    pub fn open(&mut self) -> HalResult<()> {
        use std::io::Write;
        
        let mut file = File::create(&self.file_path)
            .map_err(|e| HalError::IoError(e))?;
        
        let magic = if self.nanosecond_precision {
            PCAP_MAGIC_NUMBER_NS
        } else {
            PCAP_MAGIC_NUMBER
        };
        
        let header = [
            &magic.to_le_bytes()[..],
            &2u16.to_le_bytes()[..],
            &4u16.to_le_bytes()[..],
            &0i32.to_le_bytes()[..],
            &0u32.to_le_bytes()[..],
            &65535u32.to_le_bytes()[..],
            &(LINKTYPE_IEEE802_15_4_WITHFCS as u32).to_le_bytes()[..],
        ].concat();
        
        file.write_all(&header)
            .map_err(|e| HalError::IoError(e))?;
        
        self.file = Some(file);
        Ok(())
    }
    
    pub fn write_packet(&mut self, packet: &RawPacket) -> HalResult<()> {
        use std::io::Write;
        
        let file = self.file.as_mut()
            .ok_or(HalError::NotInitialized)?;
        
        let duration = packet.timestamp.duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0));
        
        let ts_sec = duration.as_secs() as u32;
        let ts_frac = if self.nanosecond_precision {
            duration.subsec_nanos()
        } else {
            duration.subsec_micros()
        };
        
        let packet_len = packet.data.len() as u32;
        
        let pkt_header = [
            &ts_sec.to_le_bytes()[..],
            &ts_frac.to_le_bytes()[..],
            &packet_len.to_le_bytes()[..],
            &packet_len.to_le_bytes()[..],
        ].concat();
        
        file.write_all(&pkt_header)
            .map_err(|e| HalError::IoError(e))?;
        
        file.write_all(&packet.data)
            .map_err(|e| HalError::IoError(e))?;
        
        file.flush()
            .map_err(|e| HalError::IoError(e))?;
        
        Ok(())
    }
    
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