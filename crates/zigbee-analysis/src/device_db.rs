use zigbee_core::{ParsedPacket, MacAddress};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeviceType {
    PhilipsHueBulb,
    PhilipsHueSwitch,
    IkeaTradfri,
    XiaomiSensor,
    XiaomiSwitch,
    SonoffSwitch,
    TuyaDevice,
    GenericCoordinator,
    GenericRouter,
    GenericEndDevice,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceDatabase {
    pub devices: HashMap<MacAddress, DeviceFingerprint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceFingerprint {
    pub address: MacAddress,
    pub device_type: DeviceType,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub clusters_seen: Vec<u16>,
    pub endpoints_seen: Vec<u8>,
    pub first_seen: SystemTime,
    pub last_seen: SystemTime,
    pub packet_count: u64,
}

impl DeviceDatabase {
    pub fn new() -> Self {
        Self {
            devices: HashMap::new(),
        }
    }
    
    pub fn process_packet(&mut self, parsed: &ParsedPacket) {
        let now = SystemTime::now();
        
        if let Some(src) = parsed.mac.src_addr.as_mac() {
            let fingerprint = self.devices.entry(src).or_insert_with(|| DeviceFingerprint {
                address: src,
                device_type: DeviceType::Unknown,
                manufacturer: None,
                model: None,
                clusters_seen: Vec::new(),
                endpoints_seen: Vec::new(),
                first_seen: now,
                last_seen: now,
                packet_count: 0,
            });
            
            fingerprint.last_seen = now;
            fingerprint.packet_count += 1;
            
            // Process APS layer data
            if let Some(aps) = &parsed.aps {
                // Handle source endpoint (it's an Option<u8>)
                if let Some(endpoint) = aps.src_endpoint {
                    if !fingerprint.endpoints_seen.contains(&endpoint) {
                        fingerprint.endpoints_seen.push(endpoint);
                    }
                }
                
                // Cluster ID is on the APS frame, not ZCL
                if let Some(cluster_id) = aps.cluster_id {
                    if !fingerprint.clusters_seen.contains(&cluster_id) {
                        fingerprint.clusters_seen.push(cluster_id);
                    }
                }
            }
        }
        
        // Identify devices after updating fingerprint
        if let Some(src) = parsed.mac.src_addr.as_mac() {
            if let Some(fp) = self.devices.get(&src).cloned() {
                let device_type = Self::identify_device_static(&fp);
                if let Some(device) = self.devices.get_mut(&src) {
                    device.device_type = device_type;
                }
            }
        }
    }
    
    fn identify_device_static(fingerprint: &DeviceFingerprint) -> DeviceType {
        // Philips Hue detection
        if fingerprint.clusters_seen.contains(&0x0006) // On/Off
            && fingerprint.clusters_seen.contains(&0x0008) // Level Control
            && fingerprint.clusters_seen.contains(&0x0300) { // Color Control
            return DeviceType::PhilipsHueBulb;
        }
        
        // Xiaomi sensors
        if fingerprint.clusters_seen.contains(&0x0402) // Temperature
            || fingerprint.clusters_seen.contains(&0x0405) { // Humidity
            return DeviceType::XiaomiSensor;
        }
        
        // IKEA Tradfri
        if fingerprint.endpoints_seen.contains(&1)
            && fingerprint.clusters_seen.contains(&0x0006) {
            return DeviceType::IkeaTradfri;
        }
        
        DeviceType::Unknown
    }
    
    pub fn get_device(&self, addr: &MacAddress) -> Option<&DeviceFingerprint> {
        self.devices.get(addr)
    }
    
    pub fn get_statistics(&self) -> DeviceStatistics {
        let mut stats = DeviceStatistics {
            total_devices: self.devices.len(),
            by_type: HashMap::new(),
            by_manufacturer: HashMap::new(),
        };
        
        for device in self.devices.values() {
            *stats.by_type.entry(device.device_type).or_insert(0) += 1;
            
            if let Some(ref mfr) = device.manufacturer {
                *stats.by_manufacturer.entry(mfr.clone()).or_insert(0) += 1;
            }
        }
        
        stats
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceStatistics {
    pub total_devices: usize,
    pub by_type: HashMap<DeviceType, usize>,
    pub by_manufacturer: HashMap<String, usize>,
}

impl Default for DeviceDatabase {
    fn default() -> Self {
        Self::new()
    }
}
