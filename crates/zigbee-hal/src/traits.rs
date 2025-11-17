use crate::{capabilities::DeviceCapabilities, error::HalError};
use zigbee_core::packet::RawPacket;
use async_trait::async_trait;

/// Main hardware abstraction trait for Zigbee capture devices
#[async_trait]
pub trait ZigbeeCapture: Send + Sync {
    /// Initialize the device and prepare for capture
    /// This should open connections, configure hardware, etc.
    async fn initialize(&mut self) -> Result<(), HalError>;
    
    /// Set the IEEE 802.15.4 channel to monitor (11-26 for 2.4GHz)
    async fn set_channel(&mut self, channel: u8) -> Result<(), HalError>;
    
    /// Get the currently configured channel
    fn get_channel(&self) -> Result<u8, HalError>;
    
    /// Capture the next packet (async, waits for packet)
    /// Returns when a packet is available or timeout occurs
    async fn capture_packet(&mut self) -> Result<RawPacket, HalError>;
    
    /// Non-blocking packet capture attempt
    /// Returns None immediately if no packet available
    fn try_capture_packet(&mut self) -> Result<Option<RawPacket>, HalError>;
    
    /// Get device capabilities (what features this hardware supports)
    fn capabilities(&self) -> &DeviceCapabilities;
    
    /// Human-readable device identifier
    fn device_name(&self) -> &str;
    
    /// Hardware identifier (vendor:product or similar)
    fn device_id(&self) -> String;
    
    /// Shutdown device and cleanup resources
    async fn shutdown(&mut self) -> Result<(), HalError>;
    
    /// Check if device is currently active/connected
    fn is_active(&self) -> bool;
}

/// Extended trait for devices that support packet injection (TX)
#[async_trait]
pub trait PacketInjection: ZigbeeCapture {
    /// Inject/transmit a packet
    async fn inject_packet(&mut self, packet: &RawPacket) -> Result<(), HalError>;
    
    /// Set transmission power in dBm
    async fn set_tx_power(&mut self, dbm: i8) -> Result<(), HalError>;
    
    /// Get current TX power setting
    fn get_tx_power(&self) -> Result<i8, HalError>;
    
    /// Get supported TX power range
    fn tx_power_range(&self) -> (i8, i8);
}

/// Extended trait for devices with enhanced RSSI/LQI capabilities
pub trait EnhancedMetrics: ZigbeeCapture {
    /// Get extended RSSI data with additional metadata
    fn get_rssi_extended(&self) -> Result<RssiData, HalError>;
    
    /// Get noise floor measurement
    fn get_noise_floor(&self) -> Result<i8, HalError>;
    
    /// Get channel energy scan results
    fn scan_channel_energy(&mut self, channel: u8) -> Result<u8, HalError>;
}

/// Extended trait for devices that support promiscuous mode configuration
pub trait PromiscuousMode: ZigbeeCapture {
    /// Enable/disable promiscuous mode (capture all packets regardless of addressing)
    async fn set_promiscuous(&mut self, enabled: bool) -> Result<(), HalError>;
    
    /// Check if promiscuous mode is enabled
    fn is_promiscuous(&self) -> bool;
}

/// RSSI data with extended information
#[derive(Debug, Clone, Copy)]
pub struct RssiData {
    /// RSSI value in dBm
    pub rssi_dbm: i8,
    
    /// Link Quality Indicator (0-255)
    pub lqi: u8,
    
    /// Signal-to-noise ratio if available
    pub snr: Option<f32>,
    
    /// Frequency offset in kHz if available
    pub freq_offset: Option<i16>,
}