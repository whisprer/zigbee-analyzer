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
const PCAP_MAGIC_NUMBER_NS: u32 = 0xa1b23c4d;
const PCAP_MAGIC_SWAPPED: u32 = 0xd4c3b2a1;
const PCAP_MAGIC_NS_SWAPPED: u32 = 0x4d3cb2a1;

const LINKTYPE_IEEE802_15_4: u16 = 195;
const LINKTYPE_IEEE802_15_4_WITHFCS: u16 = 230;
const LINKTYPE_IEEE802_15_4_NOFCS: u16 = 231;
const LINKTYPE_IEEE802_15_4_TAP: u16 = 283;

// FCS Radio Tap TLV types
const FCS_RADIOTAP_TLV_CHANNEL: u16 = 0;
const FCS_RADIOTAP_TLV_RSSI: u16 = 1;
const FCS_RADIOTAP_TLV_LQI: u16 = 10;
const FCS_RADIOTAP_TLV_FCS: u16 = 30;

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
        
        // Read global header (24 bytes)
        let mut header = [0u8; 24];
        file.read_exact(&mut header)
            .map_err(|e| HalError::IoError(e))?;
        
        let magic = u32::from_le_bytes([header[0], header[1], header[2], header[3]]);
        
        // Check endianness and precision
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
        
        // Parse header with correct endianness
        let read_u16 = |bytes: [u8; 2]| -> u16 {
            if swapped {
                u16::from_be_bytes(bytes)
            } else {
                u16::from_le_bytes(bytes)
            }
        };
        
        let read_u32 = |bytes: [u8; 4]| -> u32 {
            if swapped {
                u32::from_be_bytes(bytes)
            } else {
                u32::from_le_bytes(bytes)
            }
        };
        
        let _version_major = read_u16([header[4], header[5]]);
        let _version_minor = read_u16([header[6], header[7]]);
        let _this