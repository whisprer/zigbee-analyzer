pub mod mac;
pub mod network;
pub mod aps;
pub mod zcl;
pub mod error;

pub use error::ParseError;

// Re-export main parsing functions
pub use mac::parse_mac_frame;
pub use network::parse_network_frame;
pub use aps::parse_aps_frame;
pub use zcl::parse_zcl_frame;

use crate::{MacFrame, NetworkFrame, ApsFrame, ZclFrame};

/// Parsed packet at all layers
#[derive(Debug, Clone)]
pub struct ParsedPacket {
    pub mac: MacFrame,
    pub network: Option<NetworkFrame>,
    pub aps: Option<ApsFrame>,
    pub zcl: Option<ZclFrame>,
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
    })
}