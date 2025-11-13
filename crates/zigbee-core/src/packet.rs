use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Raw captured packet from hardware
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawPacket {
    /// When this packet was captured
    pub timestamp: SystemTime,
    
    /// IEEE 802.15.4 channel (11-26 for 2.4GHz)
    pub channel: u8,
    
    /// Received Signal Strength Indicator in dBm
    pub rssi: i8,
    
    /// Link Quality Indicator (0-255, higher is better)
    pub lqi: u8,
    
    /// Raw packet bytes (includes FCS/CRC at end)
    pub data: Vec<u8>,
}

impl RawPacket {
    pub fn new(channel: u8, rssi: i8, lqi: u8, data: Vec<u8>) -> Self {
        Self {
            timestamp: SystemTime::now(),
            channel,
            rssi,
            lqi,
            data,
        }
    }
    
    /// Parse this packet through all protocol layers
    pub fn parse(&self) -> Result<crate::parsers::ParsedPacket, crate::parsers::ParseError> {
        crate::parsers::parse_full_packet(&self.data)
    }
    
    /// Check if packet passes FCS (Frame Check Sequence) validation
    pub fn validate_fcs(&self) -> bool {
        if self.data.len() < 2 {
            return false;
        }
        
        // FCS is last 2 bytes - IEEE 802.15.4 CRC-16
        let payload = &self.data[..self.data.len() - 2];
        let fcs = u16::from_le_bytes([
            self.data[self.data.len() - 2],
            self.data[self.data.len() - 1],
        ]);
        
        crc16_ccitt(payload) == fcs
    }
    
    /// Get packet length without FCS
    pub fn payload_len(&self) -> usize {
        self.data.len().saturating_sub(2)
    }
}

/// CRC-16-CCITT used in IEEE 802.15.4
fn crc16_ccitt(data: &[u8]) -> u16 {
    let mut crc: u16 = 0;
    
    for &byte in data {
        crc ^= (byte as u8) << 8;
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