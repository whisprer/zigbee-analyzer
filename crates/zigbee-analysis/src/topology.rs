use zigbee_core::{MacAddress, ParsedPacket};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyMap {
    pub devices: HashMap<MacAddress, Device>,
    pub links: Vec<Link>,
    pub networks: HashMap<u16, Network>,
    pub last_updated: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub address: MacAddress,
    pub device_type: DeviceType,
    pub pan_id: Option<u16>,
    pub short_addr: Option<u16>,
    pub parent: Option<MacAddress>,
    pub children: Vec<MacAddress>,
    pub is_router: bool,
    pub is_coordinator: bool,
    pub last_seen: SystemTime,
    pub first_seen: SystemTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceType {
    Coordinator,
    Router,
    EndDevice,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub source: MacAddress,
    pub destination: MacAddress,
    pub link_quality: u8,
    pub rssi: i8,
    pub packet_count: u64,
    pub last_seen: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Network {
    pub pan_id: u16,
    pub coordinator: Option<MacAddress>,
    pub device_count: usize,
    pub first_seen: SystemTime,
    pub last_seen: SystemTime,
}

impl TopologyMap {
    pub fn new() -> Self {
        Self {
            devices: HashMap::new(),
            links: Vec::new(),
            networks: HashMap::new(),
            last_updated: SystemTime::now(),
        }
    }
    
    pub fn process_packet(&mut self, parsed: &ParsedPacket) {
        let now = SystemTime::now();
        
        // Update source device
        if let Some(src) = parsed.mac.src_addr.as_mac() {
            self.update_device(src, now);
            
            // Update PAN ID for source
            if let Some(pan_id) = parsed.mac.src_pan {
                self.update_network(pan_id, now);
                if let Some(device) = self.devices.get_mut(&src) {
                    device.pan_id = Some(pan_id);
                }
            }
        }
        
        // Update destination device
        if let Some(dst) = parsed.mac.dest_addr.as_mac() {
            self.update_device(dst, now);
            
            // Update PAN ID for destination
            if let Some(pan_id) = parsed.mac.dest_pan {
                self.update_network(pan_id, now);
                if let Some(device) = self.devices.get_mut(&dst) {
                    device.pan_id = Some(pan_id);
                }
            }
        }
        
        // Update link - handle Option rssi/lqi fields
        if let (Some(src), Some(dst)) = (
            parsed.mac.src_addr.as_mac(),
            parsed.mac.dest_addr.as_mac()
        ) {
            let rssi = parsed.rssi.unwrap_or(0);
            let lqi = parsed.lqi.unwrap_or(0);
            self.update_link(src, dst, rssi, lqi, now);
        }
        
        // Update network layer information
        if let Some(nwk) = &parsed.network {
            if let Some(src) = parsed.mac.src_addr.as_mac() {
                if let Some(device) = self.devices.get_mut(&src) {
                    device.short_addr = Some(nwk.src_addr);
                }
            }
            
            if let Some(dst) = parsed.mac.dest_addr.as_mac() {
                if let Some(device) = self.devices.get_mut(&dst) {
                    device.short_addr = Some(nwk.dest_addr);
                }
            }
        }
        
        self.last_updated = now;
    }
    
    fn update_device(&mut self, addr: MacAddress, now: SystemTime) {
        self.devices.entry(addr).or_insert_with(|| Device {
            address: addr,
            device_type: DeviceType::Unknown,
            pan_id: None,
            short_addr: None,
            parent: None,
            children: Vec::new(),
            is_router: false,
            is_coordinator: false,
            last_seen: now,
            first_seen: now,
        }).last_seen = now;
    }
    
    fn update_network(&mut self, pan_id: u16, now: SystemTime) {
        self.networks.entry(pan_id).or_insert_with(|| Network {
            pan_id,
            coordinator: None,
            device_count: 0,
            first_seen: now,
            last_seen: now,
        }).last_seen = now;
    }
    
    fn update_link(&mut self, src: MacAddress, dst: MacAddress, rssi: i8, lqi: u8, now: SystemTime) {
        // Try to find existing link
        if let Some(link) = self.links.iter_mut().find(|l| l.source == src && l.destination == dst) {
            link.packet_count += 1;
            link.last_seen = now;
            link.rssi = rssi;
            link.link_quality = lqi;
        } else {
            // Create new link
            self.links.push(Link {
                source: src,
                destination: dst,
                link_quality: lqi,
                rssi,
                packet_count: 1,
                last_seen: now,
            });
        }
    }
    
    pub fn infer_topology(&mut self) {
        // Identify routers based on link patterns
        // BORROW CHECKER FIX: Collect data first, then mutate
        let router_candidates: Vec<MacAddress> = self.links.iter()
            .map(|link| {
                let link_count = self.get_device_links(&link.source).len();
                (link.source, link_count)
            })
            .filter(|(_, count)| *count > 3)
            .map(|(addr, _)| addr)
            .collect();
        
        // Now update devices
        for addr in router_candidates {
            if let Some(src_device) = self.devices.get_mut(&addr) {
                src_device.is_router = true;
            }
        }
        
        // Update network statistics
        // BORROW CHECKER FIX: Collect all data first
        let network_updates: Vec<(u16, usize, Option<MacAddress>)> = self.networks.keys()
            .map(|&pan_id| {
                let device_count = self.get_network_devices(pan_id).len();
                let coordinator = self.get_coordinator(pan_id);
                (pan_id, device_count, coordinator)
            })
            .collect();
        
        // Then apply updates
        for (pan_id, device_count, coordinator) in network_updates {
            if let Some(network) = self.networks.get_mut(&pan_id) {
                network.device_count = device_count;
                network.coordinator = coordinator;
            }
        }
    }
    
    pub fn get_device_links(&self, addr: &MacAddress) -> Vec<&Link> {
        self.links.iter()
            .filter(|l| l.source == *addr || l.destination == *addr)
            .collect()
    }
    
    pub fn get_network_devices(&self, pan_id: u16) -> Vec<&Device> {
        self.devices.values()
            .filter(|d| d.pan_id == Some(pan_id))
            .collect()
    }
    
    pub fn get_coordinator(&self, pan_id: u16) -> Option<MacAddress> {
        self.devices.values()
            .find(|d| d.pan_id == Some(pan_id) && d.is_coordinator)
            .map(|d| d.address)
    }
    
    pub fn get_statistics(&self) -> TopologyStatistics {
        TopologyStatistics {
            total_devices: self.devices.len(),
            total_networks: self.networks.len(),
            total_links: self.links.len(),
            coordinators: self.devices.values().filter(|d| d.is_coordinator).count(),
            routers: self.devices.values().filter(|d| d.is_router).count(),
            end_devices: self.devices.values().filter(|d| 
                !d.is_coordinator && !d.is_router
            ).count(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyStatistics {
    pub total_devices: usize,
    pub total_networks: usize,
    pub total_links: usize,
    pub coordinators: usize,
    pub routers: usize,
    pub end_devices: usize,
}

impl Default for TopologyMap {
    fn default() -> Self {
        Self::new()
    }
}
