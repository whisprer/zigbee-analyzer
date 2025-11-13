use serde::{Deserialize, Serialize};

/// Zigbee Network Layer frame
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkFrame {
    pub frame_control: NwkFrameControl,
    pub dest_addr: u16,
    pub src_addr: u16,
    pub radius: u8,
    pub sequence: u8,
    pub dest_ieee: Option<[u8; 8]>,
    pub src_ieee: Option<[u8; 8]>,
    pub multicast_control: Option<u8>,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct NwkFrameControl {
    pub frame_type: NwkFrameType,
    pub protocol_version: u8,
    pub discover_route: DiscoverRoute,
    pub multicast: bool,
    pub security: bool,
    pub source_route: bool,
    pub dest_ieee_present: bool,
    pub src_ieee_present: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NwkFrameType {
    Data = 0,
    Command = 1,
    InterPan = 3,
    Reserved,
}

impl From<u8> for NwkFrameType {
    fn from(val: u8) -> Self {
        match val & 0x03 {
            0 => NwkFrameType::Data,
            1 => NwkFrameType::Command,
            3 => NwkFrameType::InterPan,
            _ => NwkFrameType::Reserved,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiscoverRoute {
    SuppressDiscovery = 0,
    EnableDiscovery = 1,
    Reserved,
}

impl From<u8> for DiscoverRoute {
    fn from(val: u8) -> Self {
        match val & 0x03 {
            0 => DiscoverRoute::SuppressDiscovery,
            1 => DiscoverRoute::EnableDiscovery,
            _ => DiscoverRoute::Reserved,
        }
    }
}

/// Zigbee Network Command types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NwkCommand {
    RouteRequest = 0x01,
    RouteReply = 0x02,
    NetworkStatus = 0x03,
    Leave = 0x04,
    RouteRecord = 0x05,
    RejoinRequest = 0x06,
    RejoinResponse = 0x07,
    LinkStatus = 0x08,
    NetworkReport = 0x09,
    NetworkUpdate = 0x0a,
    EndDeviceTimeoutRequest = 0x0b,
    EndDeviceTimeoutResponse = 0x0c,
    Unknown(u8),
}

impl From<u8> for NwkCommand {
    fn from(val: u8) -> Self {
        match val {
            0x01 => NwkCommand::RouteRequest,
            0x02 => NwkCommand::RouteReply,
            0x03 => NwkCommand::NetworkStatus,
            0x04 => NwkCommand::Leave,
            0x05 => NwkCommand::RouteRecord,
            0x06 => NwkCommand::RejoinRequest,
            0x07 => NwkCommand::RejoinResponse,
            0x08 => NwkCommand::LinkStatus,
            0x09 => NwkCommand::NetworkReport,
            0x0a => NwkCommand::NetworkUpdate,
            0x0b => NwkCommand::EndDeviceTimeoutRequest,
            0x0c => NwkCommand::EndDeviceTimeoutResponse,
            _ => NwkCommand::Unknown(val),
        }
    }
}