use crate::{
    traits::ZigbeeCapture,
    capabilities::DeviceCapabilities,
    error::{HalError, HalResult},
};
use zigbee_core::packet::RawPacket;
use async_trait::async_trait;
use std::time::SystemTime;

/// Mock device for testing without hardware
pub struct MockDevice {
    channel: u8,
    active: bool,
    capabilities: DeviceCapabilities,
    packet_queue: Vec<RawPacket>,
}

impl MockDevice {
    pub fn new() -> Self {
        Self {
            channel: 11,
            active: false,
            capabilities: DeviceCapabilities::basic_capture(),
            packet_queue: Vec::new(),
        }
    }
    
    /// Add a packet to the mock queue for testing
    pub fn queue_packet(&mut self, packet: RawPacket) {
        self.packet_queue.push(packet);
    }
    
    /// Generate a random test packet
    pub fn generate_test_packet(&self) -> RawPacket {
        RawPacket {
            timestamp: SystemTime::now(),
            channel: self.channel,
            rssi: -50,
            lqi: 200,
            data: vec![0x41, 0x88, 0x00, 0xff, 0xff, 0x00, 0x00], // Minimal beacon frame
        }
    }
}

#[async_trait]
impl ZigbeeCapture for MockDevice {
    async fn initialize(&mut self) -> HalResult<()> {
        self.active = true;
        Ok(())
    }
    
    async fn set_channel(&mut self, channel: u8) -> HalResult<()> {
        if !(11..=26).contains(&channel) {
            return Err(HalError::InvalidChannel(channel));
        }
        self.channel = channel;
        Ok(())
    }
    
    fn get_channel(&self) -> HalResult<u8> {
        Ok(self.channel)
    }
    
    async fn capture_packet(&mut self) -> HalResult<RawPacket> {
        if !self.active {
            return Err(HalError::NotInitialized);
        }
        
        // Return queued packet or generate one
        Ok(self.packet_queue.pop().unwrap_or_else(|| self.generate_test_packet()))
    }
    
    fn try_capture_packet(&mut self) -> HalResult<Option<RawPacket>> {
        if !self.active {
            return Err(HalError::NotInitialized);
        }
        Ok(self.packet_queue.pop())
    }
    
    fn capabilities(&self) -> &DeviceCapabilities {
        &self.capabilities
    }
    
    fn device_name(&self) -> &str {
        "Mock Zigbee Device"
    }
    
    fn device_id(&self) -> String {
        "mock:0000".to_string()
    }
    
    async fn shutdown(&mut self) -> HalResult<()> {
        self.active = false;
        Ok(())
    }
    
    fn is_active(&self) -> bool {
        self.active
    }
}