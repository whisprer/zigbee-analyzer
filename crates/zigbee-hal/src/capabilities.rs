use serde::{Deserialize, Serialize};

/// Describes the capabilities of a Zigbee capture device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCapabilities {
    /// Device can inject/transmit packets
    pub packet_injection: bool,
    
    /// Device supports promiscuous mode
    pub promiscuous_mode: bool,
    
    /// Device can perform energy detection scans
    pub energy_detection: bool,
    
    /// Supported IEEE 802.15.4 channels
    pub supported_channels: Vec<u8>,
    
    /// RSSI precision in dBm (e.g., 1 = ±1dBm accuracy)
    pub rssi_precision: u8,
    
    /// Maximum sample/capture rate in packets per second
    pub max_sample_rate: u32,
    
    /// Device provides timestamp information
    pub hardware_timestamps: bool,
    
    /// Device can measure noise floor
    pub noise_floor_measurement: bool,
    
    /// TX power range in dBm (min, max)
    pub tx_power_range: Option<(i8, i8)>,
    
    /// Requires external power (vs USB-powered)
    pub requires_external_power: bool,
    
    /// Buffer size for packet storage
    pub buffer_size: usize,
}

impl DeviceCapabilities {
    /// Create capabilities for a basic capture-only device (like CC2531)
    pub fn basic_capture() -> Self {
        Self {
            packet_injection: false,
            promiscuous_mode: true,
            energy_detection: false,
            supported_channels: (11..=26).collect(),
            rssi_precision: 1,
            max_sample_rate: 250, // packets per second
            hardware_timestamps: false,
            noise_floor_measurement: false,
            tx_power_range: None,
            requires_external_power: false,
            buffer_size: 256,
        }
    }
    
    /// Create capabilities for a professional device (like Silicon Labs WSTK)
    pub fn professional() -> Self {
        Self {
            packet_injection: true,
            promiscuous_mode: true,
            energy_detection: true,
            supported_channels: (11..=26).collect(),
            rssi_precision: 1,
            max_sample_rate: 1000,
            hardware_timestamps: true,
            noise_floor_measurement: true,
            tx_power_range: Some((-20, 20)),
            requires_external_power: false,
            buffer_size: 2048,
        }
    }
}