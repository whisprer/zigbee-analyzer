use zigbee_core::{MacAddress, ParsedPacket};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, Duration};

/// Network topology mapper
pub struct NetworkTopology {
    devices: HashMap<MacAddress, Device>,
    links: HashMap<LinkId, Link>,
    networks: HashMap<u16, NetworkInfo>,
    last_update: SystemTime,
}

/// A device in the network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub mac_addr: MacAddress,
    pub short_addr: Option<u16>,
    pub ieee_addr: Option<[u8; 8]>,
    pub device_type: DeviceType,
    pub first_seen: SystemTime,
    pub last_seen: SystemTime,
    pub packet_count: usize,
    pub pan_id: Option<u16>,
    pub channel: Option<u8>,
    
    // Capabilities inferred from traffic
    pub is_coordinator: bool,
    pub is_router: bool,
    pub can_route: bool,
    pub mains_powered: bool,
    
    // Network layer info
    pub nwk_addr: Option<u16>,
    pub parent_addr: Option<u16>,
    pub children: HashSet<u16>,
    
    // Statistics
    pub avg_rssi: f32,
    pub avg_lqi: f32,
    pub total_tx: usize,
    pub total_rx: usize,
    
    // Device fingerprinting
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub profiles: HashSet<u16>,
    pub clusters: HashSet<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceType {
    Unknown,
    Coordinator,
    Router,
    EndDevice,
}

/// A link between two devices
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub source: MacAddress,
    pub destination: MacAddress,
    pub link_quality: LinkQuality,
    pub packet_count: usize,
    pub first_seen: SystemTime,
    pub last_seen: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkQuality {
    pub avg_rssi: f32,
    pub avg_lqi: f32,
    pub min_rssi: i8,
    pub max_rssi: i8,
    pub packet_loss: f32,
}

/// Unique identifier for a link
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct LinkId {
    source: MacAddress,
    destination: MacAddress,
}

/// Network information (PAN)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub pan_id: u16,
    pub extended_pan_id: Option<[u8; 8]>,
    pub channel: Option<u8>,
    pub coordinator: Option<MacAddress>,
    pub device_count: usize,
    pub first_seen: SystemTime,
    pub last_seen: SystemTime,
}

impl NetworkTopology {
    pub fn new() -> Self {
        Self {
            devices: HashMap::new(),
            links: HashMap::new(),
            networks: HashMap::new(),
            last_update: SystemTime::now(),
        }
    }
    
    /// Process a parsed packet and update topology
    pub fn process_packet(&mut self, parsed: &ParsedPacket, rssi: i8, lqi: u8, channel: u8) {
        self.last_update = SystemTime::now();
        
        // Extract MAC layer info
        let src_addr = parsed.mac.src_addr;
        let dst_addr = parsed.mac.dest_addr;
        
        // Update source device
        if !src_addr.is_none() {
            self.update_device(&src_addr, rssi, lqi, channel, true);
            
            if let Some(pan_id) = parsed.mac.src_pan {
                self.update_network(pan_id);
                if let Some(device) = self.devices.get_mut(&src_addr) {
                    device.pan_id = Some(pan_id);
                }
            }
        }
        
        // Update destination device
        if !dst_addr.is_none() && !dst_addr.is_broadcast() {
            self.update_device(&dst_addr, rssi, lqi, channel, false);
            
            if let Some(pan_id) = parsed.mac.dest_pan {
                self.update_network(pan_id);
                if let Some(device) = self.devices.get_mut(&dst_addr) {
                    device.pan_id = Some(pan_id);
                }
            }
        }
        
        // Update link between devices
        if !src_addr.is_none() && !dst_addr.is_none() && !dst_addr.is_broadcast() {
            self.update_link(&src_addr, &dst_addr, rssi, lqi);
        }
        
        // Process network layer
        if let Some(nwk) = &parsed.network {
            // Update network addresses
            if !src_addr.is_none() {
                if let Some(device) = self.devices.get_mut(&src_addr) {
                    device.nwk_addr = Some(nwk.src_addr);
                    
                    // Infer device type from network address
                    if nwk.src_addr == 0x0000 {
                        device.is_coordinator = true;
                        device.device_type = DeviceType::Coordinator;
                    }
                    
                    // Store IEEE address if present
                    if let Some(ieee) = nwk.src_ieee {
                        device.ieee_addr = Some(ieee);
                    }
                }
            }
            
            if !dst_addr.is_none() {
                if let Some(device) = self.devices.get_mut(&dst_addr) {
                    device.nwk_addr = Some(nwk.dest_addr);
                    
                    if nwk.dest_addr == 0x0000 {
                        device.is_coordinator = true;
                        device.device_type = DeviceType::Coordinator;
                    }
                    
                    if let Some(ieee) = nwk.dest_ieee {
                        device.ieee_addr = Some(ieee);
                    }
                }
            }
        }
        
        // Process APS layer
        if let Some(aps) = &parsed.aps {
            if !src_addr.is_none() {
                if let Some(device) = self.devices.get_mut(&src_addr) {
                    if let Some(profile) = aps.profile_id {
                        device.profiles.insert(profile);
                    }
                    if let Some(cluster) = aps.cluster_id {
                        device.clusters.insert(cluster);
                    }
                }
            }
        }
    }
    
    /// Update or create a device
    fn update_device(&mut self, addr: &MacAddress, rssi: i8, lqi: u8, channel: u8, is_tx: bool) {
        let device = self.devices.entry(*addr).or_insert_with(|| Device {
            mac_addr: *addr,
            short_addr: match addr {
                MacAddress::Short(a) => Some(*a),
                _ => None,
            },
            ieee_addr: match addr {
                MacAddress::Extended(a) => Some(*a),
                _ => None,
            },
            device_type: DeviceType::Unknown,
            first_seen: SystemTime::now(),
            last_seen: SystemTime::now(),
            packet_count: 0,
            pan_id: None,
            channel: None,
            is_coordinator: false,
            is_router: false,
            can_route: false,
            mains_powered: false,
            nwk_addr: None,
            parent_addr: None,
            children: HashSet::new(),
            avg_rssi: rssi as f32,
            avg_lqi: lqi as f32,
            total_tx: 0,
            total_rx: 0,
            manufacturer: None,
            model: None,
            profiles: HashSet::new(),
            clusters: HashSet::new(),
        });
        
        // Update statistics
        device.last_seen = SystemTime::now();
        device.packet_count += 1;
        device.channel = Some(channel);
        
        if is_tx {
            device.total_tx += 1;
        } else {
            device.total_rx += 1;
        }
        
        // Update rolling averages
        let alpha = 0.1; // Exponential smoothing factor
        device.avg_rssi = device.avg_rssi * (1.0 - alpha) + (rssi as f32) * alpha;
        device.avg_lqi = device.avg_lqi * (1.0 - alpha) + (lqi as f32) * alpha;
    }
    
    /// Update or create a link
    fn update_link(&mut self, src: &MacAddress, dst: &MacAddress, rssi: i8, lqi: u8) {
        let link_id = LinkId {
            source: *src,
            destination: *dst,
        };
        
        let link = self.links.entry(link_id).or_insert_with(|| Link {
            source: *src,
            destination: *dst,
            link_quality: LinkQuality {
                avg_rssi: rssi as f32,
                avg_lqi: lqi as f32,
                min_rssi: rssi,
                max_rssi: rssi,
                packet_loss: 0.0,
            },
            packet_count: 0,
            first_seen: SystemTime::now(),
            last_seen: SystemTime::now(),
        });
        
        // Update link statistics
        link.last_seen = SystemTime::now();
        link.packet_count += 1;
        
        let alpha = 0.1;
        link.link_quality.avg_rssi = link.link_quality.avg_rssi * (1.0 - alpha) + (rssi as f32) * alpha;
        link.link_quality.avg_lqi = link.link_quality.avg_lqi * (1.0 - alpha) + (lqi as f32) * alpha;
        link.link_quality.min_rssi = link.link_quality.min_rssi.min(rssi);
        link.link_quality.max_rssi = link.link_quality.max_rssi.max(rssi);
    }
    
    /// Update network information
    fn update_network(&mut self, pan_id: u16) {
        let network = self.networks.entry(pan_id).or_insert_with(|| NetworkInfo {
            pan_id,
            extended_pan_id: None,
            channel: None,
            coordinator: None,
            device_count: 0,
            first_seen: SystemTime::now(),
            last_seen: SystemTime::now(),
        });
        
        network.last_seen = SystemTime::now();
    }
    
    /// Get all devices
    pub fn devices(&self) -> &HashMap<MacAddress, Device> {
        &self.devices
    }
    
    /// Get all links
    pub fn links(&self) -> &HashMap<LinkId, Link> {
        &self.links
    }
    
    /// Get all networks
    pub fn networks(&self) -> &HashMap<u16, NetworkInfo> {
        &self.networks
    }
    
    /// Get a specific device
    pub fn get_device(&self, addr: &MacAddress) -> Option<&Device> {
        self.devices.get(addr)
    }
    
    /// Get device by network address
    pub fn get_device_by_nwk_addr(&self, nwk_addr: u16) -> Option<&Device> {
        self.devices.values().find(|d| d.nwk_addr == Some(nwk_addr))
    }
    
    /// Get all devices in a network
    pub fn get_network_devices(&self, pan_id: u16) -> Vec<&Device> {
        self.devices
            .values()
            .filter(|d| d.pan_id == Some(pan_id))
            .collect()
    }
    
    /// Get coordinator for a network
    pub fn get_coordinator(&self, pan_id: u16) -> Option<&Device> {
        self.devices
            .values()
            .find(|d| d.pan_id == Some(pan_id) && d.is_coordinator)
    }
    
    /// Get routers in a network
    pub fn get_routers(&self, pan_id: u16) -> Vec<&Device> {
        self.devices
            .values()
            .filter(|d| d.pan_id == Some(pan_id) && d.is_router)
            .collect()
    }
    
    /// Get end devices in a network
    pub fn get_end_devices(&self, pan_id: u16) -> Vec<&Device> {
        self.devices
            .values()
            .filter(|d| d.pan_id == Some(pan_id) && d.device_type == DeviceType::EndDevice)
            .collect()
    }
    
    /// Get links for a specific device
    pub fn get_device_links(&self, addr: &MacAddress) -> Vec<&Link> {
        self.links
            .values()
            .filter(|l| l.source == *addr || l.destination == *addr)
            .collect()
    }
    
    /// Get neighbors (directly connected devices)
    pub fn get_neighbors(&self, addr: &MacAddress) -> Vec<&Device> {
        let mut neighbors = HashSet::new();
        
        for link in self.links.values() {
            if link.source == *addr {
                neighbors.insert(link.destination);
            }
            if link.destination == *addr {
                neighbors.insert(link.source);
            }
        }
        
        neighbors
            .iter()
            .filter_map(|addr| self.devices.get(addr))
            .collect()
    }
    
    /// Infer network topology structure
    pub fn infer_topology(&mut self) {
        // Build parent-child relationships based on traffic patterns
        for device in self.devices.values_mut() {
            if device.is_coordinator {
                device.can_route = true;
                device.mains_powered = true;
            }
        }
        
        // Detect routers (devices that forward packets for others)
        // This is a simplified heuristic
        for link in self.links.values() {
            if let Some(src_device) = self.devices.get_mut(&link.source) {
                // If a device has many outgoing links, it's likely a router
                if self.get_device_links(&link.source).len() > 3 {
                    src_device.is_router = true;
                    src_device.can_route = true;
                    if src_device.device_type == DeviceType::Unknown {
                        src_device.device_type = DeviceType::Router;
                    }
                }
            }
        }
        
        // Update network device counts
        for network in self.networks.values_mut() {
            network.device_count = self.get_network_devices(network.pan_id).len();
            network.coordinator = self.get_coordinator(network.pan_id)
                .map(|d| d.mac_addr);
        }
    }
    
    /// Clean up stale devices (not seen in timeout period)
    pub fn cleanup_stale_devices(&mut self, timeout: Duration) {
        let now = SystemTime::now();
        
        self.devices.retain(|_, device| {
            now.duration_since(device.last_seen)
                .map(|d| d < timeout)
                .unwrap_or(true)
        });
        
        self.links.retain(|_, link| {
            now.duration_since(link.last_seen)
                .map(|d| d < timeout)
                .unwrap_or(true)
        });
    }
    
    /// Get topology statistics
    pub fn get_statistics(&self) -> TopologyStatistics {
        let mut stats = TopologyStatistics {
            total_devices: self.devices.len(),
            total_links: self.links.len(),
            total_networks: self.networks.len(),
            coordinators: 0,
            routers: 0,
            end_devices: 0,
            unknown_devices: 0,
            avg_link_quality: 0.0,
            total_packets: 0,
        };
        
        for device in self.devices.values() {
            match device.device_type {
                DeviceType::Coordinator => stats.coordinators += 1,
                DeviceType::Router => stats.routers += 1,
                DeviceType::EndDevice => stats.end_devices += 1,
                DeviceType::Unknown => stats.unknown_devices += 1,
            }
            stats.total_packets += device.packet_count;
        }
        
        if !self.links.is_empty() {
            let sum_lqi: f32 = self.links.values()
                .map(|l| l.link_quality.avg_lqi)
                .sum();
            stats.avg_link_quality = sum_lqi / self.links.len() as f32;
        }
        
        stats
    }
    
    /// Export topology to DOT format (for Graphviz visualization)
    pub fn export_dot(&self) -> String {
        let mut dot = String::from("digraph ZigbeeNetwork {\n");
        dot.push_str("  rankdir=LR;\n");
        dot.push_str("  node [shape=box];\n\n");
        
        // Add nodes
        for device in self.devices.values() {
            let label = match device.nwk_addr {
                Some(addr) => format!("0x{:04x}", addr),
                None => format!("{}", device.mac_addr),
            };
            
            let color = match device.device_type {
                DeviceType::Coordinator => "red",
                DeviceType::Router => "blue",
                DeviceType::EndDevice => "green",
                DeviceType::Unknown => "gray",
            };
            
            dot.push_str(&format!(
                "  \"{}\" [label=\"{}\" color={}];\n",
                device.mac_addr, label, color
            ));
        }
        
        dot.push_str("\n");
        
        // Add edges
        for link in self.links.values() {
            let weight = (link.link_quality.avg_lqi / 255.0 * 10.0) as i32;
            dot.push_str(&format!(
                "  \"{}\" -> \"{}\" [label=\"{}\" weight={}];\n",
                link.source,
                link.destination,
                link.packet_count,
                weight
            ));
        }
        
        dot.push_str("}\n");
        dot
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyStatistics {
    pub total_devices: usize,
    pub total_links: usize,
    pub total_networks: usize,
    pub coordinators: usize,
    pub routers: usize,
    pub end_devices: usize,
    pub unknown_devices: usize,
    pub avg_link_quality: f32,
    pub total_packets: usize,
}

impl Default for NetworkTopology {
    fn default() -> Self {
        Self::new()
    }
}