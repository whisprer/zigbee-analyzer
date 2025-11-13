use serde::{Deserialize, Serialize};

/// ZCL Frame
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZclFrame {
    pub frame_control: ZclFrameControl,
    pub manufacturer_code: Option<u16>,
    pub transaction_sequence: u8,
    pub command: u8,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ZclFrameControl {
    pub frame_type: ZclFrameType,
    pub manufacturer_specific: bool,
    pub direction: ZclDirection,
    pub disable_default_response: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZclFrameType {
    Global = 0,
    ClusterSpecific = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZclDirection {
    ClientToServer = 0,
    ServerToClient = 1,
}

/// Global ZCL Commands
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZclGlobalCommand {
    ReadAttributes = 0x00,
    ReadAttributesResponse = 0x01,
    WriteAttributes = 0x02,
    WriteAttributesUndivided = 0x03,
    WriteAttributesResponse = 0x04,
    WriteAttributesNoResponse = 0x05,
    ConfigureReporting = 0x06,
    ConfigureReportingResponse = 0x07,
    ReadReportingConfiguration = 0x08,
    ReadReportingConfigurationResponse = 0x09,
    ReportAttributes = 0x0a,
    DefaultResponse = 0x0b,
    DiscoverAttributes = 0x0c,
    DiscoverAttributesResponse = 0x0d,
    ReadAttributesStructured = 0x0e,
    WriteAttributesStructured = 0x0f,
    WriteAttributesStructuredResponse = 0x10,
    DiscoverCommandsReceived = 0x11,
    DiscoverCommandsReceivedResponse = 0x12,
    DiscoverCommandsGenerated = 0x13,
    DiscoverCommandsGeneratedResponse = 0x14,
    DiscoverAttributesExtended = 0x15,
    DiscoverAttributesExtendedResponse = 0x16,
}

/// Common Zigbee Cluster IDs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClusterId {
    // General clusters
    Basic = 0x0000,
    PowerConfiguration = 0x0001,
    DeviceTemperatureConfiguration = 0x0002,
    Identify = 0x0003,
    Groups = 0x0004,
    Scenes = 0x0005,
    OnOff = 0x0006,
    OnOffSwitchConfiguration = 0x0007,
    LevelControl = 0x0008,
    Alarms = 0x0009,
    Time = 0x000a,
    
    // Lighting
    ColorControl = 0x0300,
    BallastConfiguration = 0x0301,
    
    // Measurement and Sensing
    IlluminanceMeasurement = 0x0400,
    IlluminanceLevelSensing = 0x0401,
    TemperatureMeasurement = 0x0402,
    PressureMeasurement = 0x0403,
    FlowMeasurement = 0x0404,
    RelativeHumidityMeasurement = 0x0405,
    OccupancySensing = 0x0406,
    
    // HVAC
    PumpConfigurationAndControl = 0x0200,
    ThermostatControl = 0x0201,
    FanControl = 0x0202,
    
    // Security
    IasZone = 0x0500,
    IasAce = 0x0501,
    IasWd = 0x0502,
    
    // Smart Energy
    Metering = 0x0702,
    ElectricalMeasurement = 0x0b04,
    
    Custom(u16),
}

impl From<u16> for ClusterId {
    fn from(val: u16) -> Self {
        match val {
            0x0000 => ClusterId::Basic,
            0x0001 => ClusterId::PowerConfiguration,
            0x0002 => ClusterId::DeviceTemperatureConfiguration,
            0x0003 => ClusterId::Identify,
            0x0004 => ClusterId::Groups,
            0x0005 => ClusterId::Scenes,
            0x0006 => ClusterId::OnOff,
            0x0007 => ClusterId::OnOffSwitchConfiguration,
            0x0008 => ClusterId::LevelControl,
            0x0009 => ClusterId::Alarms,
            0x000a => ClusterId::Time,
            0x0300 => ClusterId::ColorControl,
            0x0301 => ClusterId::BallastConfiguration,
            0x0400 => ClusterId::IlluminanceMeasurement,
            0x0401 => ClusterId::IlluminanceLevelSensing,
            0x0402 => ClusterId::TemperatureMeasurement,
            0x0403 => ClusterId::PressureMeasurement,
            0x0404 => ClusterId::FlowMeasurement,
            0x0405 => ClusterId::RelativeHumidityMeasurement,
            0x0406 => ClusterId::OccupancySensing,
            0x0200 => ClusterId::PumpConfigurationAndControl,
            0x0201 => ClusterId::ThermostatControl,
            0x0202 => ClusterId::FanControl,
            0x0500 => ClusterId::IasZone,
            0x0501 => ClusterId::IasAce,
            0x0502 => ClusterId::IasWd,
            0x0702 => ClusterId::Metering,
            0x0b04 => ClusterId::ElectricalMeasurement,
            _ => ClusterId::Custom(val),
        }
    }
}