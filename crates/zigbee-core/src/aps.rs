use serde::{Deserialize, Serialize};

/// Application Support Sublayer frame
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApsFrame {
    pub frame_control: ApsFrameControl,
    pub dest_endpoint: Option<u8>,
    pub cluster_id: Option<u16>,
    pub profile_id: Option<u16>,
    pub src_endpoint: Option<u8>,
    pub counter: u8,
    pub extended_header: Option<Vec<u8>>,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ApsFrameControl {
    pub frame_type: ApsFrameType,
    pub delivery_mode: DeliveryMode,
    pub ack_format: bool,
    pub security: bool,
    pub ack_request: bool,
    pub extended_header_present: bool,
    pub direction: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApsFrameType {
    Data = 0,
    Command = 1,
    Acknowledgment = 2,
    InterPan = 3,
}

impl From<u8> for ApsFrameType {
    fn from(val: u8) -> Self {
        match val & 0x03 {
            0 => ApsFrameType::Data,
            1 => ApsFrameType::Command,
            2 => ApsFrameType::Acknowledgment,
            3 => ApsFrameType::InterPan,
            _ => ApsFrameType::Data,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeliveryMode {
    Unicast = 0,
    Indirect = 1,
    Broadcast = 2,
    Group = 3,
}

impl From<u8> for DeliveryMode {
    fn from(val: u8) -> Self {
        match val & 0x03 {
            0 => DeliveryMode::Unicast,
            1 => DeliveryMode::Indirect,
            2 => DeliveryMode::Broadcast,
            3 => DeliveryMode::Group,
            _ => DeliveryMode::Unicast,
        }
    }
}

/// APS Command types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum ApsCommand {
    SkkeOne = 0x01,
    SkkeTwo = 0x02,
    SkkeThree = 0x03,
    SkkeFour = 0x04,
    TransportKey = 0x05,
    UpdateDevice = 0x06,
    RemoveDevice = 0x07,
    RequestKey = 0x08,
    SwitchKey = 0x09,
    TunnelCommand = 0x0e,
    VerifyKey = 0x0f,
    ConfirmKey = 0x10,
    Unknown(u8),
}

impl From<u8> for ApsCommand {
    fn from(val: u8) -> Self {
        match val {
            0x01 => ApsCommand::SkkeOne,
            0x02 => ApsCommand::SkkeTwo,
            0x03 => ApsCommand::SkkeThree,
            0x04 => ApsCommand::SkkeFour,
            0x05 => ApsCommand::TransportKey,
            0x06 => ApsCommand::UpdateDevice,
            0x07 => ApsCommand::RemoveDevice,
            0x08 => ApsCommand::RequestKey,
            0x09 => ApsCommand::SwitchKey,
            0x0e => ApsCommand::TunnelCommand,
            0x0f => ApsCommand::VerifyKey,
            0x10 => ApsCommand::ConfirmKey,
            _ => ApsCommand::Unknown(val),
        }
    }
}

/// Common Zigbee Profile IDs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u16)]
pub enum ProfileId {
    ZigbeeDeviceProfile = 0x0000,
    HomeAutomation = 0x0104,
    SmartEnergy = 0x0109,
    LightLink = 0xc05e,
    Green = 0xa1e0,
    Custom(u16),
}

impl From<u16> for ProfileId {
    fn from(val: u16) -> Self {
        match val {
            0x0000 => ProfileId::ZigbeeDeviceProfile,
            0x0104 => ProfileId::HomeAutomation,
            0x0109 => ProfileId::SmartEnergy,
            0xc05e => ProfileId::LightLink,
            0xa1e0 => ProfileId::Green,
            _ => ProfileId::Custom(val),
        }
    }
}

impl From<ProfileId> for u16 {
    fn from(val: ProfileId) -> u16 {
        match val {
            ProfileId::ZigbeeDeviceProfile => 0x0000,
            ProfileId::HomeAutomation => 0x0104,
            ProfileId::SmartEnergy => 0x0109,
            ProfileId::LightLink => 0xc05e,
            ProfileId::Green => 0xa1e0,
            ProfileId::Custom(id) => id,
        }
    }
}