use serde::{Deserialize, Serialize};
use std::fmt;

/// IEEE 802.15.4 MAC frame
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacFrame {
    pub frame_control: FrameControl,
    pub sequence: u8,
    pub dest_pan: Option<u16>,
    pub dest_addr: MacAddress,
    pub src_pan: Option<u16>,
    pub src_addr: MacAddress,
    pub security: Option<SecurityHeader>,
    pub payload: Vec<u8>,
    pub fcs: u16,
}

/// Frame Control field (2 bytes)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FrameControl {
    pub frame_type: FrameType,
    pub security_enabled: bool,
    pub frame_pending: bool,
    pub ack_request: bool,
    pub pan_id_compression: bool,
    pub dest_addr_mode: AddressingMode,
    pub frame_version: u8,
    pub src_addr_mode: AddressingMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FrameType {
    Beacon = 0,
    Data = 1,
    Acknowledgment = 2,
    MacCommand = 3,
    Reserved,
}

impl From<u8> for FrameType {
    fn from(val: u8) -> Self {
        match val & 0x07 {
            0 => FrameType::Beacon,
            1 => FrameType::Data,
            2 => FrameType::Acknowledgment,
            3 => FrameType::MacCommand,
            _ => FrameType::Reserved,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AddressingMode {
    None = 0,
    Reserved = 1,
    Short = 2,
    Extended = 3,
}

impl From<u8> for AddressingMode {
    fn from(val: u8) -> Self {
        match val & 0x03 {
            0 => AddressingMode::None,
            1 => AddressingMode::Reserved,
            2 => AddressingMode::Short,
            3 => AddressingMode::Extended,
            _ => AddressingMode::None,
        }
    }
}

/// MAC address (can be 16-bit short or 64-bit extended)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MacAddress {
    None,
    Short(u16),
    Extended([u8; 8]),
}

impl MacAddress {
    pub fn is_broadcast(&self) -> bool {
        matches!(self, MacAddress::Short(0xffff))
    }
    
    pub fn is_none(&self) -> bool {
        matches!(self, MacAddress::None)
    }
}

impl fmt::Display for MacAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MacAddress::None => write!(f, "None"),
            MacAddress::Short(addr) => write!(f, "{:04x}", addr),
            MacAddress::Extended(addr) => write!(
                f,
                "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                addr[0], addr[1], addr[2], addr[3], addr[4], addr[5], addr[6], addr[7]
            ),
        }
    }
}

/// Security header (if security is enabled)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityHeader {
    pub security_control: u8,
    pub frame_counter: u32,
    pub key_identifier: Option<Vec<u8>>,
}

/// MAC command types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MacCommand {
    AssociationRequest = 0x01,
    AssociationResponse = 0x02,
    DisassociationNotification = 0x03,
    DataRequest = 0x04,
    PanIdConflictNotification = 0x05,
    OrphanNotification = 0x06,
    BeaconRequest = 0x07,
    CoordinatorRealignment = 0x08,
    GtsRequest = 0x09,
    Unknown(u8),
}

impl From<u8> for MacCommand {
    fn from(val: u8) -> Self {
        match val {
            0x01 => MacCommand::AssociationRequest,
            0x02 => MacCommand::AssociationResponse,
            0x03 => MacCommand::DisassociationNotification,
            0x04 => MacCommand::DataRequest,
            0x05 => MacCommand::PanIdConflictNotification,
            0x06 => MacCommand::OrphanNotification,
            0x07 => MacCommand::BeaconRequest,
            0x08 => MacCommand::CoordinatorRealignment,
            0x09 => MacCommand::GtsRequest,
            _ => MacCommand::Unknown(val),
        }
    }
}