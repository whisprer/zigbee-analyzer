use zigbee_core::{ParsedPacket, FrameType, NwkFrameType, ApsFrameType, MacAddress};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, Duration};

/// Traffic statistics tracker
pub struct TrafficStatistics {
    // Global counters
    pub total_packets: u64,
    pub total_bytes: u64,
    pub start_time: SystemTime,
    pub last_update: SystemTime,
    
    // Frame type counters
    pub frame_types: FrameTypeCounters,
    
    // Channel statistics
    pub channel_stats: HashMap<u8, ChannelStats>,
    
    // Temporal statistics
    pub temporal: TemporalStats,
    
    // Per-device statistics
    pub device_stats: HashMap<MacAddress, DeviceStats>,
    
    // Protocol layer statistics
    pub protocol_stats: ProtocolStats,
    
    // Error tracking
    pub error_stats: ErrorStats,
}

/// Frame type counters
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FrameTypeCounters {
    pub beacon: u64,
    pub data: u64,
    pub acknowledgment: u64,
    pub mac_command: u64,
    pub reserved: u64,
}

/// Channel-specific statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelStats {
    pub channel: u8,
    pub packet_count: u64,
    pub byte_count: u64,
    pub avg_rssi: f32,
    pub min_rssi: i8,
    pub max_rssi: i8,
    pub avg_lqi: f32,
    pub utilization: f32, // Estimated channel utilization (0-1)
    pub first_seen: SystemTime,
    pub last_seen: SystemTime,
}

/// Temporal statistics (time-based patterns)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalStats {
    // Packets per second tracking
    pub current_pps: f32,
    pub peak_pps: f32,
    pub avg_pps: f32,
    
    // Bytes per second tracking
    pub current_bps: f32,
    pub peak_bps: f32,
    pub avg_bps: f32,
    
    // Time windows for rate calculation
    last_second_packets: Vec<(SystemTime, u64)>,
    last_second_bytes: Vec<(SystemTime, u64)>,
    
    // Packet inter-arrival times
    pub avg_interarrival_ms: f32,
    pub min_interarrival_ms: f32,
    pub max_interarrival_ms: f32,
    last_packet_time: Option<SystemTime>,
}

/// Per-device statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceStats {
    pub mac_addr: MacAddress,
    pub tx_packets: u64,
    pub rx_packets: u64,
    pub tx_bytes: u64,
    pub rx_bytes: u64,
    pub avg_rssi: f32,
    pub avg_lqi: f32,
    pub first_seen: SystemTime,
    pub last_seen: SystemTime,
    pub active_time: Duration,
}

/// Protocol layer statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProtocolStats {
    // MAC layer
    pub mac_with_security: u64,
    pub mac_with_ack_request: u64,
    pub mac_pan_id_compressed: u64,
    
    // Network layer
    pub nwk_packets: u64,
    pub nwk_data: u64,
    pub nwk_commands: u64,
    pub nwk_interpan: u64,
    pub nwk_with_security: u64,
    pub nwk_with_source_route: u64,
    
    // APS layer
    pub aps_packets: u64,
    pub aps_data: u64,
    pub aps_commands: u64,
    pub aps_acks: u64,
    pub aps_with_security: u64,
    
    // ZCL layer
    pub zcl_packets: u64,
    pub zcl_global_commands: u64,
    pub zcl_cluster_commands: u64,
    
    // Profile distribution
    pub profiles: HashMap<u16, u64>,
    
    // Cluster distribution
    pub clusters: HashMap<u16, u64>,
}

/// Error and anomaly statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ErrorStats {
    pub parse_errors: u64,
    pub fcs_errors: u64,
    pub malformed_packets: u64,
    pub unknown_frame_types: u64,
    pub security_errors: u64,
}

impl TrafficStatistics {
    pub fn new() -> Self {
        let now = SystemTime::now();
        
        Self {
            total_packets: 0,
            total_bytes: 0,
            start_time: now,
            last_update: now,
            frame_types: FrameTypeCounters::default(),
            channel_stats: HashMap::new(),
            temporal: TemporalStats::new(),
            device_stats: HashMap::new(),
            protocol_stats: ProtocolStats::default(),
            error_stats: ErrorStats::default(),
        }
    }
    
    /// Process a raw packet
    pub fn process_raw_packet(&mut self, data: &[u8], rssi: i8, lqi: u8, channel: u8) {
        self.total_packets += 1;
        self.total_bytes += data.len() as u64;
        self.last_update = SystemTime::now();
        
        // Update channel statistics
        self.update_channel_stats(channel, data.len(), rssi, lqi);
        
        // Update temporal statistics
        self.temporal.update(data.len());
    }
    
    /// Process a parsed packet
    pub fn process_parsed_packet(&mut self, parsed: &ParsedPacket, rssi: i8, lqi: u8, channel: u8) {
        let data_len = parsed.mac.payload.len();
        
        // Update raw packet stats
        self.process_raw_packet(&parsed.mac.payload, rssi, lqi, channel);
        
        // Update frame type counters
        match parsed.mac.frame_control.frame_type {
            FrameType::Beacon => self.frame_types.beacon += 1,
            FrameType::Data => self.frame_types.data += 1,
            FrameType::Acknowledgment => self.frame_types.acknowledgment += 1,
            FrameType::MacCommand => self.frame_types.mac_command += 1,
            FrameType::Reserved => self.frame_types.reserved += 1,
        }
        
        // Update MAC layer stats
        if parsed.mac.frame_control.security_enabled {
            self.protocol_stats.mac_with_security += 1;
        }
        if parsed.mac.frame_control.ack_request {
            self.protocol_stats.mac_with_ack_request += 1;
        }
        if parsed.mac.frame_control.pan_id_compression {
            self.protocol_stats.mac_pan_id_compressed += 1;
        }
        
        // Update device statistics
        if !parsed.mac.src_addr.is_none() {
            self.update_device_stats(&parsed.mac.src_addr, data_len, rssi, lqi, true);
        }
        if !parsed.mac.dest_addr.is_none() && !parsed.mac.dest_addr.is_broadcast() {
            self.update_device_stats(&parsed.mac.dest_addr, data_len, rssi, lqi, false);
        }
        
        // Process network layer
        if let Some(nwk) = &parsed.network {
            self.protocol_stats.nwk_packets += 1;
            
            match nwk.frame_control.frame_type {
                NwkFrameType::Data => self.protocol_stats.nwk_data += 1,
                NwkFrameType::Command => self.protocol_stats.nwk_commands += 1,
                NwkFrameType::InterPan => self.protocol_stats.nwk_interpan += 1,
                _ => {}
            }
            
            if nwk.frame_control.security {
                self.protocol_stats.nwk_with_security += 1;
            }
            if nwk.frame_control.source_route {
                self.protocol_stats.nwk_with_source_route += 1;
            }
        }
        
        // Process APS layer
        if let Some(aps) = &parsed.aps {
            self.protocol_stats.aps_packets += 1;
            
            match aps.frame_control.frame_type {
                ApsFrameType::Data => self.protocol_stats.aps_data += 1,
                ApsFrameType::Command => self.protocol_stats.aps_commands += 1,
                ApsFrameType::Acknowledgment => self.protocol_stats.aps_acks += 1,
                _ => {}
            }
            
            if aps.frame_control.security {
                self.protocol_stats.aps_with_security += 1;
            }
            
            // Track profiles
            if let Some(profile) = aps.profile_id {
                *self.protocol_stats.profiles.entry(profile).or_insert(0) += 1;
            }
            
            // Track clusters
            if let Some(cluster) = aps.cluster_id {
                *self.protocol_stats.clusters.entry(cluster).or_insert(0) += 1;
            }
        }
        
        // Process ZCL layer
        if let Some(_zcl) = &parsed.zcl {
            self.protocol_stats.zcl_packets += 1;
            
            // Could add more detailed ZCL stats here
        }
    }
    
    /// Record a parse error
    pub fn record_parse_error(&mut self) {
        self.error_stats.parse_errors += 1;
    }
    
    /// Record an FCS error
    pub fn record_fcs_error(&mut self) {
        self.error_stats.fcs_errors += 1;
    }
    
    /// Update channel statistics
    fn update_channel_stats(&mut self, channel: u8, packet_len: usize, rssi: i8, lqi: u8) {
        let now = SystemTime::now();
        
        let stats = self.channel_stats.entry(channel).or_insert_with(|| ChannelStats {
            channel,
            packet_count: 0,
            byte_count: 0,
            avg_rssi: rssi as f32,
            min_rssi: rssi,
            max_rssi: rssi,
            avg_lqi: lqi as f32,
            utilization: 0.0,
            first_seen: now,
            last_seen: now,
        });
        
        stats.packet_count += 1;
        stats.byte_count += packet_len as u64;
        stats.last_seen = now;
        
        // Update rolling averages
        let alpha = 0.1;
        stats.avg_rssi = stats.avg_rssi * (1.0 - alpha) + (rssi as f32) * alpha;
        stats.avg_lqi = stats.avg_lqi * (1.0 - alpha) + (lqi as f32) * alpha;
        stats.min_rssi = stats.min_rssi.min(rssi);
        stats.max_rssi = stats.max_rssi.max(rssi);
        
        // Estimate channel utilization (simplified)
        if let Ok(duration) = now.duration_since(stats.first_seen) {
            let duration_secs = duration.as_secs_f32();
            if duration_secs > 0.0 {
                // Rough estimate: assume 250kbps max throughput
                let max_bytes = 250_000.0 / 8.0 * duration_secs;
                stats.utilization = (stats.byte_count as f32 / max_bytes).min(1.0);
            }
        }
    }
    
    /// Update device statistics
    fn update_device_stats(&mut self, addr: &MacAddress, packet_len: usize, rssi: i8, lqi: u8, is_tx: bool) {
        let now = SystemTime::now();
        
        let stats = self.device_stats.entry(*addr).or_insert_with(|| DeviceStats {
            mac_addr: *addr,
            tx_packets: 0,
            rx_packets: 0,
            tx_bytes: 0,
            rx_bytes: 0,
            avg_rssi: rssi as f32,
            avg_lqi: lqi as f32,
            first_seen: now,
            last_seen: now,
            active_time: Duration::from_secs(0),
        });
        
        if is_tx {
            stats.tx_packets += 1;
            stats.tx_bytes += packet_len as u64;
        } else {
            stats.rx_packets += 1;
            stats.rx_bytes += packet_len as u64;
        }
        
        // Update rolling averages
        let alpha = 0.1;
        stats.avg_rssi = stats.avg_rssi * (1.0 - alpha) + (rssi as f32) * alpha;
        stats.avg_lqi = stats.avg_lqi * (1.0 - alpha) + (lqi as f32) * alpha;
        
        // Update active time
        if let Ok(duration) = now.duration_since(stats.first_seen) {
            stats.active_time = duration;
        }
        
        stats.last_seen = now;
    }
    
    /// Get packets per second
    pub fn packets_per_second(&self) -> f32 {
        self.temporal.current_pps
    }
    
    /// Get bytes per second
    pub fn bytes_per_second(&self) -> f32 {
        self.temporal.current_bps
    }
    
    /// Get overall statistics summary
    pub fn get_summary(&self) -> StatisticsSummary {
        let uptime = SystemTime::now()
            .duration_since(self.start_time)
            .unwrap_or(Duration::from_secs(0));
        
        let avg_packet_size = if self.total_packets > 0 {
            self.total_bytes as f32 / self.total_packets as f32
        } else {
            0.0
        };
        
        StatisticsSummary {
            total_packets: self.total_packets,
            total_bytes: self.total_bytes,
            uptime_seconds: uptime.as_secs(),
            avg_packet_size,
            packets_per_second: self.temporal.current_pps,
            bytes_per_second: self.temporal.current_bps,
            peak_pps: self.temporal.peak_pps,
            peak_bps: self.temporal.peak_bps,
            unique_devices: self.device_stats.len(),
            active_channels: self.channel_stats.len(),
            beacon_frames: self.frame_types.beacon,
            data_frames: self.frame_types.data,
            ack_frames: self.frame_types.acknowledgment,
            command_frames: self.frame_types.mac_command,
            parse_errors: self.error_stats.parse_errors,
            fcs_errors: self.error_stats.fcs_errors,
        }
    }
    
    /// Get top N devices by packet count
    pub fn get_top_devices(&self, n: usize) -> Vec<(&MacAddress, &DeviceStats)> {
        let mut devices: Vec<_> = self.device_stats.iter().collect();
        devices.sort_by(|a, b| {
            (b.1.tx_packets + b.1.rx_packets).cmp(&(a.1.tx_packets + a.1.rx_packets))
        });
        devices.into_iter().take(n).collect()
    }
    
    /// Get top N profiles by usage
    pub fn get_top_profiles(&self, n: usize) -> Vec<(u16, u64)> {
        let mut profiles: Vec<_> = self.protocol_stats.profiles.iter()
            .map(|(k, v)| (*k, *v))
            .collect();
        profiles.sort_by(|a, b| b.1.cmp(&a.1));
        profiles.into_iter().take(n).collect()
    }
    
    /// Get top N clusters by usage
    pub fn get_top_clusters(&self, n: usize) -> Vec<(u16, u64)> {
        let mut clusters: Vec<_> = self.protocol_stats.clusters.iter()
            .map(|(k, v)| (*k, *v))
            .collect();
        clusters.sort_by(|a, b| b.1.cmp(&a.1));
        clusters.into_iter().take(n).collect()
    }
    
    /// Get channel with most activity
    pub fn get_busiest_channel(&self) -> Option<(u8, &ChannelStats)> {
        self.channel_stats.iter()
            .max_by_key(|(_, stats)| stats.packet_count)
            .map(|(ch, stats)| (*ch, stats))
    }
    
    /// Reset all statistics
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

impl TemporalStats {
    fn new() -> Self {
        Self {
            current_pps: 0.0,
            peak_pps: 0.0,
            avg_pps: 0.0,
            current_bps: 0.0,
            peak_bps: 0.0,
            avg_bps: 0.0,
            last_second_packets: Vec::new(),
            last_second_bytes: Vec::new(),
            avg_interarrival_ms: 0.0,
            min_interarrival_ms: f32::MAX,
            max_interarrival_ms: 0.0,
            last_packet_time: None,
        }
    }
    
    fn update(&mut self, packet_len: usize) {
        let now = SystemTime::now();
        
        // Calculate inter-arrival time
        if let Some(last_time) = self.last_packet_time {
            if let Ok(duration) = now.duration_since(last_time) {
                let ms = duration.as_secs_f32() * 1000.0;
                
                // Update inter-arrival statistics
                let alpha = 0.1;
                if self.avg_interarrival_ms == 0.0 {
                    self.avg_interarrival_ms = ms;
                } else {
                    self.avg_interarrival_ms = self.avg_interarrival_ms * (1.0 - alpha) + ms * alpha;
                }
                
                self.min_interarrival_ms = self.min_interarrival_ms.min(ms);
                self.max_interarrival_ms = self.max_interarrival_ms.max(ms);
            }
        }
        self.last_packet_time = Some(now);
        
        // Track packets in last second
        self.last_second_packets.push((now, 1));
        self.last_second_bytes.push((now, packet_len as u64));
        
        // Remove entries older than 1 second
        let cutoff = now - Duration::from_secs(1);
        self.last_second_packets.retain(|(time, _)| *time > cutoff);
        self.last_second_bytes.retain(|(time, _)| *time > cutoff);
        
        // Calculate current rates
        let packets_in_window: u64 = self.last_second_packets.iter().map(|(_, c)| c).sum();
        let bytes_in_window: u64 = self.last_second_bytes.iter().map(|(_, c)| c).sum();
        
        self.current_pps = packets_in_window as f32;
        self.current_bps = bytes_in_window as f32;
        
        // Update peaks
        if self.current_pps > self.peak_pps {
            self.peak_pps = self.current_pps;
        }
        if self.current_bps > self.peak_bps {
            self.peak_bps = self.current_bps;
        }
        
        // Update averages (exponential smoothing)
        let alpha = 0.05;
        self.avg_pps = self.avg_pps * (1.0 - alpha) + self.current_pps * alpha;
        self.avg_bps = self.avg_bps * (1.0 - alpha) + self.current_bps * alpha;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticsSummary {
    pub total_packets: u64,
    pub total_bytes: u64,
    pub uptime_seconds: u64,
    pub avg_packet_size: f32,
    pub packets_per_second: f32,
    pub bytes_per_second: f32,
    pub peak_pps: f32,
    pub peak_bps: f32,
    pub unique_devices: usize,
    pub active_channels: usize,
    pub beacon_frames: u64,
    pub data_frames: u64,
    pub ack_frames: u64,
    pub command_frames: u64,
    pub parse_errors: u64,
    pub fcs_errors: u64,
}

impl Default for TrafficStatistics {
    fn default() -> Self {
        Self::new()
    }
}