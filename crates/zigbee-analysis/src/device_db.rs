use zigbee_core::{ParsedPacket, MacAddress, ProfileId, ClusterId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, Duration};

/// Device fingerprinting database
pub struct DeviceDatabase {
    // Known devices
    devices: HashMap<MacAddress, DeviceFingerprint>,
    
    // Manufacturer database
    manufacturers: ManufacturerDb,
    
    // Device signatures for identification
    signatures: Vec<DeviceSignature>,
    
    // Learning mode
    learning_mode: bool,
}

/// Complete fingerprint for a device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceFingerprint {
    pub mac_addr: MacAddress,
    pub ieee_addr: Option<[u8; 8]>,
    pub short_addr: Option<u16>,
    
    // Identification
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub device_type: DeviceType,
    pub confidence: f32, // 0.0 - 1.0
    
    // Capabilities
    pub capabilities: DeviceCapabilities,
    
    // Behavior profile
    pub behavior: BehaviorProfile,
    
    // Protocol details
    pub protocol: ProtocolProfile,
    
    // Power characteristics
    pub power_source: PowerSource,
    
    // Firmware
    pub firmware_version: Option<String>,
    pub stack_version: Option<String>,
    
    // Statistics
    pub first_seen: SystemTime,
    pub last_seen: SystemTime,
    pub packet_count: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceType {
    Unknown,
    Coordinator,
    Router,
    EndDevice,
    
    // Lighting
    OnOffLight,
    DimmableLight,
    ColorLight,
    ExtendedColorLight,
    
    // Switches/Controls
    OnOffSwitch,
    DimmerSwitch,
    ColorController,
    RemoteControl,
    
    // Sensors
    TemperatureSensor,
    HumiditySensor,
    OccupancySensor,
    ContactSensor,
    MotionSensor,
    WaterLeakSensor,
    SmokeSensor,
    
    // HVAC
    Thermostat,
    FanController,
    
    // Security
    DoorLock,
    WindowCovering,
    WarningDevice,
    
    // Smart Home
    SmartPlug,
    PowerMeter,
    
    // Other
    Gateway,
    Bridge,
    Repeater,
    Unknown_,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCapabilities {
    pub can_route: bool,
    pub rx_on_when_idle: bool,
    pub supports_binding: bool,
    pub supports_groups: bool,
    pub supports_scenes: bool,
    pub supports_reporting: bool,
    pub supports_ota: bool,
    
    // Zigbee features
    pub supports_touchlink: bool,
    pub supports_green_power: bool,
    
    // Detected from traffic
    pub max_buffer_size: Option<usize>,
    pub max_neighbors: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorProfile {
    // Communication patterns
    pub typical_peers: HashSet<MacAddress>,
    pub primary_controller: Option<MacAddress>,
    pub reports_to: HashSet<MacAddress>,
    
    // Timing
    pub avg_report_interval: Option<Duration>,
    pub active_time_of_day: Vec<bool>, // 24 hour profile
    pub duty_cycle: f32, // 0.0 - 1.0
    
    // Traffic patterns
    pub avg_packet_size: f32,
    pub tx_rx_ratio: f32,
    pub burst_sender: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolProfile {
    // Supported profiles
    pub profiles: HashSet<u16>,
    pub primary_profile: Option<u16>,
    
    // Supported clusters (server side)
    pub server_clusters: HashSet<u16>,
    
    // Supported clusters (client side)
    pub client_clusters: HashSet<u16>,
    
    // Endpoints
    pub endpoints: HashSet<u8>,
    
    // Protocol quirks
    pub uses_broadcast: bool,
    pub uses_multicast: bool,
    pub uses_source_routing: bool,
    
    // Security
    pub always_encrypted: bool,
    pub encryption_rate: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PowerSource {
    Unknown,
    Mains,          // Always powered
    Battery,        // Battery only
    Rechargeable,   // Rechargeable battery
    Hybrid,         // Battery + mains
}

/// Manufacturer database (OUI lookups)
struct ManufacturerDb {
    oui_map: HashMap<u32, String>, // 24-bit OUI -> Manufacturer
}

/// Device signature for identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceSignature {
    pub name: String,
    pub manufacturer: String,
    pub device_type: DeviceType,
    
    // Matching criteria
    pub profiles: Vec<u16>,
    pub clusters: Vec<u16>,
    pub endpoint_count: Option<usize>,
    pub model_id: Option<String>,
    pub manufacturer_code: Option<u16>,
    
    // Behavioral markers
    pub typical_packet_size: Option<(usize, usize)>, // (min, max)
    pub report_interval: Option<Duration>,
    pub power_source: Option<PowerSource>,
    
    // Match confidence weight
    pub weight: f32,
}

impl DeviceDatabase {
    pub fn new() -> Self {
        let mut db = Self {
            devices: HashMap::new(),
            manufacturers: ManufacturerDb::new(),
            signatures: Vec::new(),
            learning_mode: false,
        };
        
        // Load built-in signatures
        db.load_builtin_signatures();
        
        db
    }
    
    /// Process a packet and update device fingerprint
    pub fn process_packet(&mut self, parsed: &ParsedPacket) {
        let src = parsed.mac.src_addr;
        
        if src.is_none() || src.is_broadcast() {
            return;
        }
        
        let now = SystemTime::now();
        
        // Get or create fingerprint
        let fingerprint = self.devices.entry(src).or_insert_with(|| DeviceFingerprint {
            mac_addr: src,
            ieee_addr: None,
            short_addr: match src {
                MacAddress::Short(addr) => Some(addr),
                _ => None,
            },
            manufacturer: None,
            model: None,
            device_type: DeviceType::Unknown,
            confidence: 0.0,
            capabilities: DeviceCapabilities::default(),
            behavior: BehaviorProfile::default(),
            protocol: ProtocolProfile::default(),
            power_source: PowerSource::Unknown,
            firmware_version: None,
            stack_version: None,
            first_seen: now,
            last_seen: now,
            packet_count: 0,
        });
        
        fingerprint.last_seen = now;
        fingerprint.packet_count += 1;
        
        // Extract IEEE address if available
        if let MacAddress::Extended(ieee) = src {
            fingerprint.ieee_addr = Some(ieee);
            
            // Try to identify manufacturer from OUI
            if fingerprint.manufacturer.is_none() {
                let oui = ((ieee[0] as u32) << 16) | ((ieee[1] as u32) << 8) | (ieee[2] as u32);
                if let Some(manufacturer) = self.manufacturers.lookup(oui) {
                    fingerprint.manufacturer = Some(manufacturer);
                }
            }
        }
        
        // Update network address
        if let Some(nwk) = &parsed.network {
            fingerprint.short_addr = Some(nwk.src_addr);
            
            // Extract IEEE from network layer if present
            if let Some(ieee) = nwk.src_ieee {
                fingerprint.ieee_addr = Some(ieee);
            }
            
            // Detect coordinator
            if nwk.src_addr == 0x0000 {
                fingerprint.device_type = DeviceType::Coordinator;
                fingerprint.capabilities.can_route = true;
                fingerprint.capabilities.rx_on_when_idle = true;
                fingerprint.power_source = PowerSource::Mains;
            }
        }
        
        // Update protocol profile from APS layer
        if let Some(aps) = &parsed.aps {
            if let Some(profile) = aps.profile_id {
                fingerprint.protocol.profiles.insert(profile);
                
                // Set primary profile if not set
                if fingerprint.protocol.primary_profile.is_none() {
                    fingerprint.protocol.primary_profile = Some(profile);
                }
            }
            
            if let Some(cluster) = aps.cluster_id {
                // Determine if this is server or client side based on direction
                if aps.frame_control.direction == zigbee_core::zcl::ZclDirection::ServerToClient {
                    fingerprint.protocol.server_clusters.insert(cluster);
                } else {
                    fingerprint.protocol.client_clusters.insert(cluster);
                }
            }
            
            if let Some(ep) = aps.src_endpoint {
                fingerprint.protocol.endpoints.insert(ep);
            }
        }
        
        // Update behavior profile
        let dst = parsed.mac.dest_addr;
        if !dst.is_none() && !dst.is_broadcast() {
            fingerprint.behavior.typical_peers.insert(dst);
        }
        
        // Update packet size tracking
        let packet_size = parsed.mac.payload.len() as f32;
        let alpha = 0.1;
        if fingerprint.behavior.avg_packet_size == 0.0 {
            fingerprint.behavior.avg_packet_size = packet_size;
        } else {
            fingerprint.behavior.avg_packet_size = 
                fingerprint.behavior.avg_packet_size * (1.0 - alpha) + packet_size * alpha;
        }
        
        // Update encryption tracking
        if parsed.mac.frame_control.security_enabled {
            fingerprint.protocol.encryption_rate = 
                (fingerprint.protocol.encryption_rate * (fingerprint.packet_count - 1) as f32 + 1.0) 
                / fingerprint.packet_count as f32;
        } else {
            fingerprint.protocol.encryption_rate = 
                (fingerprint.protocol.encryption_rate * (fingerprint.packet_count - 1) as f32) 
                / fingerprint.packet_count as f32;
        }
        
        if fingerprint.protocol.encryption_rate > 0.95 {
            fingerprint.protocol.always_encrypted = true;
        }
        
        // Try to identify device type
        self.identify_device(fingerprint);
    }
    
    /// Attempt to identify device type based on collected data
    fn identify_device(&self, fingerprint: &mut DeviceFingerprint) {
        let mut best_match: Option<&DeviceSignature> = None;
        let mut best_score = 0.0f32;
        
        for signature in &self.signatures {
            let score = self.match_signature(fingerprint, signature);
            
            if score > best_score {
                best_score = score;
                best_match = Some(signature);
            }
        }
        
        if let Some(signature) = best_match {
            if best_score > 0.6 {
                fingerprint.device_type = signature.device_type;
                fingerprint.confidence = best_score;
                
                if fingerprint.manufacturer.is_none() {
                    fingerprint.manufacturer = Some(signature.manufacturer.clone());
                }
                
                if fingerprint.model.is_none() {
                    fingerprint.model = Some(signature.name.clone());
                }
            }
        }
        
        // Fallback heuristics if no signature match
        if fingerprint.device_type == DeviceType::Unknown {
            self.heuristic_identification(fingerprint);
        }
    }
    
    /// Calculate match score for a signature
    fn match_signature(&self, fingerprint: &DeviceFingerprint, signature: &DeviceSignature) -> f32 {
        let mut score = 0.0f32;
        let mut max_score = 0.0f32;
        
        // Profile matching (weight: 30)
        max_score += 30.0;
        let profile_matches = signature.profiles.iter()
            .filter(|p| fingerprint.protocol.profiles.contains(p))
            .count();
        if !signature.profiles.is_empty() {
            score += (profile_matches as f32 / signature.profiles.len() as f32) * 30.0;
        }
        
        // Cluster matching (weight: 40)
        max_score += 40.0;
        let cluster_matches = signature.clusters.iter()
            .filter(|c| fingerprint.protocol.server_clusters.contains(c) 
                     || fingerprint.protocol.client_clusters.contains(c))
            .count();
        if !signature.clusters.is_empty() {
            score += (cluster_matches as f32 / signature.clusters.len() as f32) * 40.0;
        }
        
        // Endpoint count (weight: 10)
        if let Some(expected_ep_count) = signature.endpoint_count {
            max_score += 10.0;
            let actual_ep_count = fingerprint.protocol.endpoints.len();
            if actual_ep_count == expected_ep_count {
                score += 10.0;
            }
        }
        
        // Power source (weight: 10)
        if let Some(expected_power) = signature.power_source {
            max_score += 10.0;
            if fingerprint.power_source == expected_power {
                score += 10.0;
            }
        }
        
        // Packet size (weight: 10)
        if let Some((min_size, max_size)) = signature.typical_packet_size {
            max_score += 10.0;
            let size = fingerprint.behavior.avg_packet_size as usize;
            if size >= min_size && size <= max_size {
                score += 10.0;
            }
        }
        
        // Normalize score
        if max_score > 0.0 {
            (score / max_score) * signature.weight
        } else {
            0.0
        }
    }
    
    /// Heuristic device identification when no signature matches
    fn heuristic_identification(&self, fingerprint: &mut DeviceFingerprint) {
        // Check for lighting devices
        if fingerprint.protocol.server_clusters.contains(&0x0006) { // OnOff cluster
            if fingerprint.protocol.server_clusters.contains(&0x0300) { // Color control
                fingerprint.device_type = DeviceType::ColorLight;
                fingerprint.confidence = 0.7;
            } else if fingerprint.protocol.server_clusters.contains(&0x0008) { // Level control
                fingerprint.device_type = DeviceType::DimmableLight;
                fingerprint.confidence = 0.7;
            } else {
                fingerprint.device_type = DeviceType::OnOffLight;
                fingerprint.confidence = 0.7;
            }
            return;
        }
        
        // Check for switches/controllers
        if fingerprint.protocol.client_clusters.contains(&0x0006) {
            if fingerprint.protocol.client_clusters.contains(&0x0300) {
                fingerprint.device_type = DeviceType::ColorController;
                fingerprint.confidence = 0.6;
            } else if fingerprint.protocol.client_clusters.contains(&0x0008) {
                fingerprint.device_type = DeviceType::DimmerSwitch;
                fingerprint.confidence = 0.6;
            } else {
                fingerprint.device_type = DeviceType::OnOffSwitch;
                fingerprint.confidence = 0.6;
            }
            return;
        }
        
        // Check for sensors
        if fingerprint.protocol.server_clusters.contains(&0x0402) { // Temperature
            fingerprint.device_type = DeviceType::TemperatureSensor;
            fingerprint.confidence = 0.7;
            return;
        }
        if fingerprint.protocol.server_clusters.contains(&0x0405) { // Humidity
            fingerprint.device_type = DeviceType::HumiditySensor;
            fingerprint.confidence = 0.7;
            return;
        }
        if fingerprint.protocol.server_clusters.contains(&0x0406) { // Occupancy
            fingerprint.device_type = DeviceType::OccupancySensor;
            fingerprint.confidence = 0.7;
            return;
        }
        
        // Check for HVAC
        if fingerprint.protocol.server_clusters.contains(&0x0201) { // Thermostat
            fingerprint.device_type = DeviceType::Thermostat;
            fingerprint.confidence = 0.8;
            return;
        }
        
        // Check for security devices
        if fingerprint.protocol.server_clusters.contains(&0x0101) { // Door lock
            fingerprint.device_type = DeviceType::DoorLock;
            fingerprint.confidence = 0.8;
            return;
        }
        
        // Infer from behavior
        if fingerprint.capabilities.can_route {
            fingerprint.device_type = DeviceType::Router;
            fingerprint.confidence = 0.5;
        } else if fingerprint.power_source == PowerSource::Battery {
            fingerprint.device_type = DeviceType::EndDevice;
            fingerprint.confidence = 0.4;
        }
    }
    
    /// Load built-in device signatures
    fn load_builtin_signatures(&mut self) {
        // Philips Hue lights
        self.signatures.push(DeviceSignature {
            name: "Philips Hue Color Bulb".to_string(),
            manufacturer: "Philips".to_string(),
            device_type: DeviceType::ExtendedColorLight,
            profiles: vec![0x0104], // Home Automation
            clusters: vec![0x0000, 0x0003, 0x0004, 0x0005, 0x0006, 0x0008, 0x0300], // Basic, Identify, Groups, Scenes, OnOff, Level, Color
            endpoint_count: Some(1),
            model_id: None,
            manufacturer_code: Some(0x100b),
            typical_packet_size: Some((20, 80)),
            report_interval: None,
            power_source: Some(PowerSource::Mains),
            weight: 1.0,
        });
        
        // IKEA Tradfri
        self.signatures.push(DeviceSignature {
            name: "IKEA Tradfri Bulb".to_string(),
            manufacturer: "IKEA".to_string(),
            device_type: DeviceType::DimmableLight,
            profiles: vec![0x0104],
            clusters: vec![0x0000, 0x0003, 0x0004, 0x0005, 0x0006, 0x0008],
            endpoint_count: Some(1),
            model_id: None,
            manufacturer_code: Some(0x117c),
            typical_packet_size: Some((15, 60)),
            report_interval: None,
            power_source: Some(PowerSource::Mains),
            weight: 1.0,
        });
        
        // Xiaomi Aqara sensors
        self.signatures.push(DeviceSignature {
            name: "Xiaomi Aqara Temperature Sensor".to_string(),
            manufacturer: "Xiaomi".to_string(),
            device_type: DeviceType::TemperatureSensor,
            profiles: vec![0x0104],
            clusters: vec![0x0000, 0x0001, 0x0402, 0x0405], // Basic, Power, Temp, Humidity
            endpoint_count: Some(1),
            model_id: None,
            manufacturer_code: Some(0x115f),
            typical_packet_size: Some((10, 50)),
            report_interval: Some(Duration::from_secs(300)),
            power_source: Some(PowerSource::Battery),
            weight: 1.0,
        });
        
        // Generic smart plug
        self.signatures.push(DeviceSignature {
            name: "Smart Plug".to_string(),
            manufacturer: "Generic".to_string(),
            device_type: DeviceType::SmartPlug,
            profiles: vec![0x0104],
            clusters: vec![0x0000, 0x0003, 0x0004, 0x0005, 0x0006, 0x0702], // + Metering
            endpoint_count: Some(1),
            model_id: None,
            manufacturer_code: None,
            typical_packet_size: Some((20, 70)),
            report_interval: None,
            power_source: Some(PowerSource::Mains),
            weight: 0.7,
        });
        
        // Add more signatures as needed...
    }
    
    /// Get fingerprint for a device
    pub fn get_fingerprint(&self, device: &MacAddress) -> Option<&DeviceFingerprint> {
        self.devices.get(device)
    }
    
    /// Get all fingerprints
    pub fn get_all_fingerprints(&self) -> &HashMap<MacAddress, DeviceFingerprint> {
        &self.devices
    }
    
    /// Get devices by type
    pub fn get_devices_by_type(&self, device_type: DeviceType) -> Vec<&DeviceFingerprint> {
        self.devices.values()
            .filter(|d| d.device_type == device_type)
            .collect()
    }
    
    /// Get devices by manufacturer
    pub fn get_devices_by_manufacturer(&self, manufacturer: &str) -> Vec<&DeviceFingerprint> {
        self.devices.values()
            .filter(|d| d.manufacturer.as_deref() == Some(manufacturer))
            .collect()
    }
    
    /// Get unidentified devices
    pub fn get_unidentified_devices(&self) -> Vec<&DeviceFingerprint> {
        self.devices.values()
            .filter(|d| d.device_type == DeviceType::Unknown || d.confidence < 0.5)
            .collect()
    }
    
    /// Get statistics
    pub fn get_statistics(&self) -> DeviceStatistics {
        let mut stats = DeviceStatistics {
            total_devices: self.devices.len(),
            identified_devices: 0,
            unidentified_devices: 0,
            by_type: HashMap::new(),
            by_manufacturer: HashMap::new(),
            avg_confidence: 0.0,
        };
        
        let mut total_confidence = 0.0;
        
        for device in self.devices.values() {
            if device.confidence >= 0.5 {
                stats.identified_devices += 1;
            } else {
                stats.unidentified_devices += 1;
            }
            
            *stats.by_type.entry(device.device_type).or_insert(0) += 1;
            
            if let Some(ref mfr) = device.manufacturer {
                *stats.by_manufacturer.entry(mfr.clone()).or_insert(0) += 1;
            }
            
            total_confidence += device.confidence;
        }
        
        if !self.devices.is_empty() {
            stats.avg_confidence = total_confidence / self.devices.len() as f32;
        }
        
        stats
    }
    
    /// Add a custom signature
    pub fn add_signature(&mut self, signature: DeviceSignature) {
        self.signatures.push(signature);
    }
    
    /// Export fingerprints to JSON
    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.devices)
    }
    
    /// Import fingerprints from JSON
    pub fn import_json(&mut self, json: &str) -> Result<(), serde_json::Error> {
        let devices: HashMap<MacAddress, DeviceFingerprint> = serde_json::from_str(json)?;
        self.devices.extend(devices);
        Ok(())
    }
}

impl ManufacturerDb {
    fn new() -> Self {
        let mut oui_map = HashMap::new();
        
        // Load common Zigbee manufacturers (OUI prefixes)
        oui_map.insert(0x00124b, "Texas Instruments".to_string());
        oui_map.insert(0x000d6f, "Philips".to_string());
        oui_map.insert(0x001788, "Philips".to_string());
        oui_map.insert(0x00178a, "Philips".to_string());
        oui_map.insert(0x001fee, "GE".to_string());
        oui_map.insert(0x00137a, "Belkin".to_string());
        oui_map.insert(0x000b57, "Samsung".to_string());
        oui_map.insert(0x00158d, "Xiaomi".to_string());
        oui_map.insert(0x04cf8c, "Xiaomi".to_string());
        oui_map.insert(0x086bd7, "Xiaomi".to_string());
        oui_map.insert(0x001d0f, "Ember (Silicon Labs)".to_string());
        oui_map.insert(0x000b91, "Jennic (NXP)".to_string());
        oui_map.insert(0x000f0d, "Microchip".to_string());
        oui_map.insert(0x90fd9f, "IKEA".to_string());
        oui_map.insert(0x001b57, "OSRAM".to_string());
        oui_map.insert(0x0004a3, "OSRAM".to_string());
        oui_map.insert(0x00129b, "Bosch".to_string());
        oui_map.insert(0x001e5e, "Yale".to_string());
        oui_map.insert(0x000d6f, "Lutron".to_string());
        oui_map.insert(0x001cf0, "Dresden Elektronik".to_string());
        
        Self { oui_map }
    }
    
    fn lookup(&self, oui: u32) -> Option<String> {
        self.oui_map.get(&oui).cloned()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceStatistics {
    pub total_devices: usize,
    pub identified_devices: usize,
    pub unidentified_devices: usize,
    pub by_type: HashMap<DeviceType, usize>,
    pub by_manufacturer: HashMap<String, usize>,
    pub avg_confidence: f32,
}

impl Default for DeviceCapabilities {
    fn default() -> Self {
        Self {
            can_route: false,
            rx_on_when_idle: false,
            supports_binding: false,
            supports_groups: false,
            supports_scenes: false,
            supports_reporting: false,
            supports_ota: false,
            supports_touchlink: false,
            supports_green_power: false,
            max_buffer_size: None,
            max_neighbors: None,
        }
    }
}

impl Default for BehaviorProfile {
    fn default() -> Self {
        Self {
            typical_peers: HashSet::new(),
            primary_controller: None,
            reports_to: HashSet::new(),
            avg_report_interval: None,
            active_time_of_day: vec![false; 24],
            duty_cycle: 0.0,
            avg_packet_size: 0.0,
            tx_rx_ratio: 0.0,
            burst_sender: false,
        }
    }
}

impl Default for ProtocolProfile {
    fn default() -> Self {
        Self {
            profiles: HashSet::new(),
            primary_profile: None,
            server_clusters: HashSet::new(),
            client_clusters: HashSet::new(),
            endpoints: HashSet::new(),
            uses_broadcast: false,
            uses_multicast: false,
            uses_source_routing: false,
            always_encrypted: false,
            encryption_rate: 0.0,
        }
    }
}

impl Default for DeviceDatabase {
    fn default() -> Self {
        Self::new()
    }
}