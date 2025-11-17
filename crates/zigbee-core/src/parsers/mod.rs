pub mod error;
pub mod mac;
pub mod network;
pub mod aps;
pub mod zcl;

pub use error::ParseError;

// Import the parsing functions
use crate::parsers::mac::parse_mac_frame;
use crate::parsers::network::parse_network_frame;
use crate::parsers::aps::parse_aps_frame;
use crate::parsers::zcl::parse_zcl_frame;

// Import the frame types
use crate::ieee802154::MacFrame;
use crate::network::NetworkFrame;
use crate::aps::ApsFrame;
use crate::zcl::ZclFrame;

/// Parsed packet at all layers with metadata
#[derive(Debug, Clone)]
pub struct ParsedPacket {
    pub mac: MacFrame,
    pub network: Option<NetworkFrame>,
    pub aps: Option<ApsFrame>,
    pub zcl: Option<ZclFrame>,
    pub rssi: Option<i8>,
    pub lqi: Option<u8>,
    pub timestamp: Option<std::time::SystemTime>,
}

impl ParsedPacket {
    /// Create a new ParsedPacket with just MAC layer
    pub fn new(mac: MacFrame) -> Self {
        Self {
            mac,
            network: None,
            aps: None,
            zcl: None,
            rssi: None,
            lqi: None,
            timestamp: Some(std::time::SystemTime::now()),
        }
    }
    
    /// Set RSSI value
    pub fn with_rssi(mut self, rssi: i8) -> Self {
        self.rssi = Some(rssi);
        self
    }
    
    /// Set LQI value
    pub fn with_lqi(mut self, lqi: u8) -> Self {
        self.lqi = Some(lqi);
        self
    }
    
    /// Set timestamp
    pub fn with_timestamp(mut self, timestamp: std::time::SystemTime) -> Self {
        self.timestamp = Some(timestamp);
        self
    }
}

/// Parse a complete Zigbee packet through all layers
pub fn parse_full_packet(data: &[u8]) -> error::ParseResult<ParsedPacket> {
    // Parse MAC layer
    let mac = parse_mac_frame(data)?;
    
    // Try to parse Network layer
    let network = if mac.frame_control.frame_type == crate::ieee802154::FrameType::Data 
        && !mac.payload.is_empty() {
        parse_network_frame(&mac.payload).ok()
    } else {
        None
    };
    
    // Try to parse APS layer
    let aps = if let Some(ref nwk) = network {
        if nwk.frame_control.frame_type == crate::network::NwkFrameType::Data
            && !nwk.payload.is_empty() {
            parse_aps_frame(&nwk.payload).ok()
        } else {
            None
        }
    } else {
        None
    };
    
    // Try to parse ZCL layer
    let zcl = if let Some(ref aps_frame) = aps {
        if aps_frame.frame_control.frame_type == crate::aps::ApsFrameType::Data
            && !aps_frame.payload.is_empty() {
            parse_zcl_frame(&aps_frame.payload).ok()
        } else {
            None
        }
    } else {
        None
    };
    
    Ok(ParsedPacket {
        mac,
        network,
        aps,
        zcl,
        rssi: None,
        lqi: None,
        timestamp: Some(std::time::SystemTime::now()),
    })
}
