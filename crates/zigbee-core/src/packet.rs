use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Raw packet captured from hardware
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawPacket {
    pub timestamp: SystemTime,
    pub channel: u8,
    pub rssi: i8,
    pub lqi: u8,
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
    
    /// Parse the raw packet into structured format
    pub fn parse(&self) -> Result<crate::parsers::ParsedPacket, crate::parsers::ParseError> {
        crate::parsers::parse_full_packet(&self.data)
    }
}