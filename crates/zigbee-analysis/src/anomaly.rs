#![allow(dead_code)]
use zigbee_core::{ParsedPacket, MacAddress, FrameType};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque, HashSet};
use std::time::{SystemTime, Duration};

/// Anomaly detection engine
pub struct AnomalyDetector {
    // Sub-detectors
    traffic_detector: TrafficAnomalyDetector,
    security_detector: SecurityAnomalyDetector,
    behavior_detector: BehaviorAnomalyDetector,
    protocol_detector: ProtocolAnomalyDetector,
    
    // Detected anomalies
    anomalies: Vec<Anomaly>,
    
    // Configuration
    config: DetectorConfig,
    
    // Statistics
    start_time: SystemTime,
    total_packets_processed: u64,
}

/// Configuration for anomaly detection
#[derive(Debug, Clone)]
pub struct DetectorConfig {
    pub traffic_spike_threshold: f32,      // Multiplier over baseline (e.g., 3.0 = 300%)
    pub behavior_change_threshold: f32,    // Sensitivity to behavioral changes
    pub min_baseline_samples: usize,       // Minimum packets to establish baseline
    pub anomaly_retention_hours: u64,      // How long to keep anomaly records
    pub enable_traffic_detection: bool,
    pub enable_security_detection: bool,
    pub enable_behavior_detection: bool,
    pub enable_protocol_detection: bool,
}

/// Detected anomaly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    pub id: u64,
    pub timestamp: SystemTime,
    pub anomaly_type: AnomalyType,
    pub severity: Severity,
    pub description: String,
    pub affected_device: Option<MacAddress>,
    pub evidence: Vec<String>,
    pub confidence: f32, // 0.0 - 1.0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Hash)]
pub enum AnomalyType {
    // Traffic anomalies
    TrafficSpike,
    TrafficDrop,
    UnusualRate,
    FloodAttack,
    
    // Security anomalies
    ReplayAttack,
    AddressSpoofing,
    UnauthorizedDevice,
    SequenceAnomaly,
    SecurityBypass,
    
    // Behavioral anomalies
    UnexpectedCommunication,
    RoleChange,
    PatternDeviation,
    ChannelHopping,
    
    // Protocol anomalies
    MalformedPacket,
    ProtocolViolation,
    InvalidSequence,
    UnknownCommand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// Traffic anomaly detector
struct TrafficAnomalyDetector {
    // Packet rate tracking
    packet_history: VecDeque<(SystemTime, u64)>,
    baseline_pps: f32,
    current_pps: f32,
    
    // Per-device traffic
    device_rates: HashMap<MacAddress, DeviceTrafficProfile>,
    
    // Flood detection
    last_packet_time: Option<SystemTime>,
    rapid_packet_count: u32,
}

#[derive(Debug, Clone)]
struct DeviceTrafficProfile {
    baseline_tx_rate: f32,
    baseline_rx_rate: f32,
    current_tx_rate: f32,
    current_rx_rate: f32,
    last_tx: Option<SystemTime>,
    last_rx: Option<SystemTime>,
    tx_history: VecDeque<SystemTime>,
    rx_history: VecDeque<SystemTime>,
}

/// Security anomaly detector
struct SecurityAnomalyDetector {
    // Sequence number tracking
    device_sequences: HashMap<MacAddress, SequenceTracker>,
    
    // Known devices (whitelist)
    known_devices: HashSet<MacAddress>,
    
    // Replay detection
    recent_packets: VecDeque<PacketFingerprint>,
    
    // Address spoofing detection
    mac_to_nwk_mapping: HashMap<MacAddress, u16>,
}

#[derive(Debug, Clone)]
struct SequenceTracker {
    last_sequence: u8,
    sequence_gaps: Vec<u8>,
    out_of_order_count: u32,
    last_update: SystemTime,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct PacketFingerprint {
    src: MacAddress,
    dst: MacAddress,
    sequence: u8,
    payload_hash: u64,
    timestamp: SystemTime,
}

/// Behavioral anomaly detector
struct BehaviorAnomalyDetector {
    // Device behavior profiles
    device_behaviors: HashMap<MacAddress, BehaviorProfile>,
    
    // Communication patterns
    communication_graph: HashMap<(MacAddress, MacAddress), CommunicationPattern>,
}

#[derive(Debug, Clone)]
struct BehaviorProfile {
    typical_peers: HashSet<MacAddress>,
    typical_channels: HashSet<u8>,
    typical_frame_types: HashMap<FrameType, u64>,
    avg_packet_size: f32,
    active_hours: Vec<bool>, // 24 hour profile
    last_channel: Option<u8>,
    role_stable: bool,
}

#[derive(Debug, Clone)]
struct CommunicationPattern {
    packet_count: u64,
    avg_interval: Duration,
    first_seen: SystemTime,
    last_seen: SystemTime,
}

/// Protocol anomaly detector
struct ProtocolAnomalyDetector {
    // Malformed packet tracking
    malformed_count: u64,
    
    // Protocol state machines per device
    device_states: HashMap<MacAddress, ProtocolState>,
}

#[derive(Debug, Clone)]
struct ProtocolState {
    expected_next: Vec<FrameType>,
    violation_count: u32,
    last_frame_type: Option<FrameType>,
}

impl AnomalyDetector {
    pub fn new() -> Self {
        Self::with_config(DetectorConfig::default())
    }
    
    pub fn with_config(config: DetectorConfig) -> Self {
        Self {
            traffic_detector: TrafficAnomalyDetector::new(),
            security_detector: SecurityAnomalyDetector::new(),
            behavior_detector: BehaviorAnomalyDetector::new(),
            protocol_detector: ProtocolAnomalyDetector::new(),
            anomalies: Vec::new(),
            config,
            start_time: SystemTime::now(),
            total_packets_processed: 0,
        }
    }
    
    /// Process a parsed packet for anomaly detection
    pub fn process_packet(&mut self, parsed: &ParsedPacket, _rssi: i8, channel: u8) {
        self.total_packets_processed += 1;
        
        let mut detected_anomalies = Vec::new();
        
        // Traffic anomaly detection
        if self.config.enable_traffic_detection {
            if let Some(anomaly) = self.traffic_detector.check_packet(parsed, &self.config) {
                detected_anomalies.push(anomaly);
            }
        }
        
        // Security anomaly detection
        if self.config.enable_security_detection {
            if let Some(anomaly) = self.security_detector.check_packet(parsed) {
                detected_anomalies.push(anomaly);
            }
        }
        
        // Behavioral anomaly detection
        if self.config.enable_behavior_detection {
            if let Some(anomaly) = self.behavior_detector.check_packet(parsed, channel) {
                detected_anomalies.push(anomaly);
            }
        }
        
        // Protocol anomaly detection
        if self.config.enable_protocol_detection {
            if let Some(anomaly) = self.protocol_detector.check_packet(parsed) {
                detected_anomalies.push(anomaly);
            }
        }
        
        // Add detected anomalies
        for mut anomaly in detected_anomalies {
            anomaly.id = self.anomalies.len() as u64;
            self.anomalies.push(anomaly);
        }
        
        // Cleanup old anomalies
        self.cleanup_old_anomalies();
    }
    
    /// Get all detected anomalies
    pub fn get_anomalies(&self) -> &[Anomaly] {
        &self.anomalies
    }
    
    /// Get anomalies by type
    pub fn get_anomalies_by_type(&self, anomaly_type: AnomalyType) -> Vec<&Anomaly> {
        self.anomalies.iter()
            .filter(|a| a.anomaly_type == anomaly_type)
            .collect()
    }
    
    /// Get anomalies by severity
    pub fn get_anomalies_by_severity(&self, min_severity: Severity) -> Vec<&Anomaly> {
        self.anomalies.iter()
            .filter(|a| a.severity >= min_severity)
            .collect()
    }
    
    /// Get anomalies for a specific device
    pub fn get_device_anomalies(&self, device: &MacAddress) -> Vec<&Anomaly> {
        self.anomalies.iter()
            .filter(|a| a.affected_device.as_ref() == Some(device))
            .collect()
    }
    
    /// Get recent anomalies (last N)
    pub fn get_recent_anomalies(&self, count: usize) -> Vec<&Anomaly> {
        self.anomalies.iter()
            .rev()
            .take(count)
            .collect()
    }
    
    /// Get anomaly statistics
    pub fn get_statistics(&self) -> AnomalyStatistics {
        let mut stats = AnomalyStatistics {
            total_anomalies: self.anomalies.len(),
            critical: 0,
            high: 0,
            medium: 0,
            low: 0,
            by_type: HashMap::new(),
            packets_processed: self.total_packets_processed,
            anomaly_rate: 0.0,
        };
        
        for anomaly in &self.anomalies {
            match anomaly.severity {
                Severity::Critical => stats.critical += 1,
                Severity::High => stats.high += 1,
                Severity::Medium => stats.medium += 1,
                Severity::Low => stats.low += 1,
            }
            
            *stats.by_type.entry(anomaly.anomaly_type).or_insert(0) += 1;
        }
        
        if self.total_packets_processed > 0 {
            stats.anomaly_rate = self.anomalies.len() as f32 / self.total_packets_processed as f32;
        }
        
        stats
    }
    
    /// Clear all anomalies
    pub fn clear_anomalies(&mut self) {
        self.anomalies.clear();
    }
    
    /// Register a known/trusted device
    pub fn register_trusted_device(&mut self, device: MacAddress) {
        self.security_detector.known_devices.insert(device);
    }
    
    fn cleanup_old_anomalies(&mut self) {
        let cutoff = SystemTime::now() - Duration::from_secs(self.config.anomaly_retention_hours * 3600);
        self.anomalies.retain(|a| a.timestamp > cutoff);
    }
}

impl TrafficAnomalyDetector {
    fn new() -> Self {
        Self {
            packet_history: VecDeque::new(),
            baseline_pps: 0.0,
            current_pps: 0.0,
            device_rates: HashMap::new(),
            last_packet_time: None,
            rapid_packet_count: 0,
        }
    }
    
    fn check_packet(&mut self, parsed: &ParsedPacket, config: &DetectorConfig) -> Option<Anomaly> {
        let now = SystemTime::now();
        
        // Update packet history
        self.packet_history.push_back((now, 1));
        
        // Keep only last 60 seconds
        let cutoff = now - Duration::from_secs(60);
        while self.packet_history.front().map(|(t, _)| *t < cutoff).unwrap_or(false) {
            self.packet_history.pop_front();
        }
        
        // Calculate current PPS
        self.current_pps = self.packet_history.len() as f32;
        
        // Update baseline (exponential moving average)
        if self.packet_history.len() >= config.min_baseline_samples {
            let alpha = 0.05;
            if self.baseline_pps == 0.0 {
                self.baseline_pps = self.current_pps;
            } else {
                self.baseline_pps = self.baseline_pps * (1.0 - alpha) + self.current_pps * alpha;
            }
        }
        
        // Check for traffic spike
        if self.baseline_pps > 0.0 && self.current_pps > self.baseline_pps * config.traffic_spike_threshold {
            return Some(Anomaly {
                id: 0,
                timestamp: now,
                anomaly_type: AnomalyType::TrafficSpike,
                severity: if self.current_pps > self.baseline_pps * 5.0 {
                    Severity::High
                } else {
                    Severity::Medium
                },
                description: format!(
                    "Traffic spike detected: {:.1} pps (baseline: {:.1} pps)",
                    self.current_pps, self.baseline_pps
                ),
                affected_device: None,
                evidence: vec![
                    format!("Current rate: {:.1} pps", self.current_pps),
                    format!("Baseline rate: {:.1} pps", self.baseline_pps),
                    format!("Spike factor: {:.1}x", self.current_pps / self.baseline_pps),
                ],
                confidence: 0.9,
            });
        }
        
        // Check for traffic drop
        if self.baseline_pps > 10.0 && self.current_pps < self.baseline_pps * 0.2 {
            return Some(Anomaly {
                id: 0,
                timestamp: now,
                anomaly_type: AnomalyType::TrafficDrop,
                severity: Severity::Medium,
                description: format!(
                    "Significant traffic drop: {:.1} pps (baseline: {:.1} pps)",
                    self.current_pps, self.baseline_pps
                ),
                affected_device: None,
                evidence: vec![
                    format!("Current rate: {:.1} pps", self.current_pps),
                    format!("Baseline rate: {:.1} pps", self.baseline_pps),
                ],
                confidence: 0.85,
            });
        }
        
        // Flood detection (rapid packets from same source)
        if let Some(last_time) = self.last_packet_time {
            if let Ok(duration) = now.duration_since(last_time) {
                if duration < Duration::from_millis(10) {
                    self.rapid_packet_count += 1;
                    
                    if self.rapid_packet_count > 50 {
                        return Some(Anomaly {
                            id: 0,
                            timestamp: now,
                            anomaly_type: AnomalyType::FloodAttack,
                            severity: Severity::Critical,
                            description: "Possible flood attack detected: rapid packet transmission".to_string(),
                            affected_device: Some(parsed.mac.src_addr),
                            evidence: vec![
                                format!("Rapid packet count: {}", self.rapid_packet_count),
                                format!("Inter-packet interval: < 10ms"),
                            ],
                            confidence: 0.95,
                        });
                    }
                } else {
                    self.rapid_packet_count = 0;
                }
            }
        }
        self.last_packet_time = Some(now);
        
        None
    }
}

impl SecurityAnomalyDetector {
    fn new() -> Self {
        Self {
            device_sequences: HashMap::new(),
            known_devices: HashSet::new(),
            recent_packets: VecDeque::new(),
            mac_to_nwk_mapping: HashMap::new(),
        }
    }
    
    fn check_packet(&mut self, parsed: &ParsedPacket) -> Option<Anomaly> {
        let now = SystemTime::now();
        let src = parsed.mac.src_addr;
        
        // Check for unauthorized device
        if !src.is_none() && !src.is_broadcast() && !self.known_devices.is_empty() {
            if !self.known_devices.contains(&src) {
                self.known_devices.insert(src);
                return Some(Anomaly {
                    id: 0,
                    timestamp: now,
                    anomaly_type: AnomalyType::UnauthorizedDevice,
                    severity: Severity::High,
                    description: format!("New/unauthorized device detected: {}", src),
                    affected_device: Some(src),
                    evidence: vec![
                        format!("Device address: {}", src),
                        "Device not in trusted list".to_string(),
                    ],
                    confidence: 0.8,
                });
            }
        }
        
        // Check sequence numbers
        let sequence = parsed.mac.sequence;
        let tracker = self.device_sequences.entry(src).or_insert_with(|| SequenceTracker {
            last_sequence: sequence,
            sequence_gaps: Vec::new(),
            out_of_order_count: 0,
            last_update: now,
        });
        
        // Detect sequence anomalies
        let expected_seq = tracker.last_sequence.wrapping_add(1);
        if sequence != expected_seq && sequence != tracker.last_sequence {
            tracker.out_of_order_count += 1;
            
            if tracker.out_of_order_count > 10 {
                return Some(Anomaly {
                    id: 0,
                    timestamp: now,
                    anomaly_type: AnomalyType::SequenceAnomaly,
                    severity: Severity::Medium,
                    description: format!("Sequence number anomaly from {}", src),
                    affected_device: Some(src),
                    evidence: vec![
                        format!("Expected: {}, Got: {}", expected_seq, sequence),
                        format!("Out-of-order count: {}", tracker.out_of_order_count),
                    ],
                    confidence: 0.7,
                });
            }
        }
        
        tracker.last_sequence = sequence;
        tracker.last_update = now;
        
        // Replay detection (simplified)
        let fingerprint = PacketFingerprint {
            src,
            dst: parsed.mac.dest_addr,
            sequence,
            payload_hash: simple_hash(&parsed.mac.payload),
            timestamp: now,
        };
        
        // Check if we've seen this exact packet recently
        for recent in &self.recent_packets {
            if recent.src == fingerprint.src 
                && recent.dst == fingerprint.dst
                && recent.payload_hash == fingerprint.payload_hash
                && recent.sequence == fingerprint.sequence {
                
                if let Ok(duration) = now.duration_since(recent.timestamp) {
                    if duration < Duration::from_secs(5) {
                        return Some(Anomaly {
                            id: 0,
                            timestamp: now,
                            anomaly_type: AnomalyType::ReplayAttack,
                            severity: Severity::Critical,
                            description: format!("Possible replay attack from {}", src),
                            affected_device: Some(src),
                            evidence: vec![
                                "Duplicate packet detected".to_string(),
                                format!("Time since original: {:.1}s", duration.as_secs_f32()),
                            ],
                            confidence: 0.85,
                        });
                    }
                }
            }
        }
        
        self.recent_packets.push_back(fingerprint);
        
        // Keep only last 1000 packets
        if self.recent_packets.len() > 1000 {
            self.recent_packets.pop_front();
        }
        
        // Check address spoofing (MAC/NWK mismatch)
        if let Some(nwk) = &parsed.network {
            if !src.is_none() {
                if let Some(&known_nwk) = self.mac_to_nwk_mapping.get(&src) {
                    if known_nwk != nwk.src_addr && nwk.src_addr != 0xffff {
                        return Some(Anomaly {
                            id: 0,
                            timestamp: now,
                            anomaly_type: AnomalyType::AddressSpoofing,
                            severity: Severity::Critical,
                            description: format!("Address spoofing detected: MAC {} claims NWK 0x{:04x} but known as 0x{:04x}",
                                src, nwk.src_addr, known_nwk),
                            affected_device: Some(src),
                            evidence: vec![
                                format!("MAC address: {}", src),
                                format!("Claimed NWK: 0x{:04x}", nwk.src_addr),
                                format!("Known NWK: 0x{:04x}", known_nwk),
                            ],
                            confidence: 0.9,
                        });
                    }
                } else {
                    self.mac_to_nwk_mapping.insert(src, nwk.src_addr);
                }
            }
        }
        
        None
    }
}

impl BehaviorAnomalyDetector {
    fn new() -> Self {
        Self {
            device_behaviors: HashMap::new(),
            communication_graph: HashMap::new(),
        }
    }
    
    fn check_packet(&mut self, parsed: &ParsedPacket, channel: u8) -> Option<Anomaly> {
        let now = SystemTime::now();
        let src = parsed.mac.src_addr;
        let dst = parsed.mac.dest_addr;
        
        if src.is_none() {
            return None;
        }
        
        // Get or create behavior profile
        let profile = self.device_behaviors.entry(src).or_insert_with(|| BehaviorProfile {
            typical_peers: HashSet::new(),
            typical_channels: HashSet::new(),
            typical_frame_types: HashMap::new(),
            avg_packet_size: 0.0,
            active_hours: vec![false; 24],
            last_channel: None,
            role_stable: true,
        });
        
        // Check for unexpected peer communication
        if !dst.is_none() && !dst.is_broadcast() {
            if profile.typical_peers.len() > 5 && !profile.typical_peers.contains(&dst) {
                profile.typical_peers.insert(dst);
                return Some(Anomaly {
                    id: 0,
                    timestamp: now,
                    anomaly_type: AnomalyType::UnexpectedCommunication,
                    severity: Severity::Low,
                    description: format!("{} communicating with new peer {}", src, dst),
                    affected_device: Some(src),
                    evidence: vec![
                        format!("Source: {}", src),
                        format!("New peer: {}", dst),
                    ],
                    confidence: 0.6,
                });
            }
            profile.typical_peers.insert(dst);
        }
        
        // Check for channel hopping
        if let Some(last_ch) = profile.last_channel {
            if last_ch != channel {
                return Some(Anomaly {
                    id: 0,
                    timestamp: now,
                    anomaly_type: AnomalyType::ChannelHopping,
                    severity: Severity::Medium,
                    description: format!("{} changed channel: {} → {}", src, last_ch, channel),
                    affected_device: Some(src),
                    evidence: vec![
                        format!("Previous channel: {}", last_ch),
                        format!("New channel: {}", channel),
                    ],
                    confidence: 0.8,
                });
            }
        }
        profile.last_channel = Some(channel);
        profile.typical_channels.insert(channel);
        
        // Track frame types
        let frame_type = parsed.mac.frame_control.frame_type;
        *profile.typical_frame_types.entry(frame_type).or_insert(0) += 1;
        
        None
    }
}

impl ProtocolAnomalyDetector {
    fn new() -> Self {
        Self {
            malformed_count: 0,
            device_states: HashMap::new(),
        }
    }
    
    fn check_packet(&mut self, parsed: &ParsedPacket) -> Option<Anomaly> {
        // Check for protocol violations (simplified)
        let src = parsed.mac.src_addr;
        
        if src.is_none() {
            return None;
        }
        
        let state = self.device_states.entry(src).or_insert_with(|| ProtocolState {
            expected_next: Vec::new(),
            violation_count: 0,
            last_frame_type: None,
        });
        
        let frame_type = parsed.mac.frame_control.frame_type;
        
        // Simple state machine check (simplified)
        if let Some(last_type) = state.last_frame_type {
            // Example: Data frames shouldn't immediately follow beacons in most cases
            if last_type == FrameType::Beacon && frame_type == FrameType::Data {
                state.violation_count += 1;
            }
        }
        
        state.last_frame_type = Some(frame_type);
        
        if state.violation_count > 20 {
            state.violation_count = 0;
            return Some(Anomaly {
                id: 0,
                timestamp: SystemTime::now(),
                anomaly_type: AnomalyType::ProtocolViolation,
                severity: Severity::Low,
                description: format!("Protocol violations detected from {}", src),
                affected_device: Some(src),
                evidence: vec!["Multiple protocol state violations".to_string()],
                confidence: 0.5,
            });
        }
        
        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyStatistics {
    pub total_anomalies: usize,
    pub critical: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
    pub by_type: HashMap<AnomalyType, usize>,
    pub packets_processed: u64,
    pub anomaly_rate: f32,
}

impl Default for DetectorConfig {
    fn default() -> Self {
        Self {
            traffic_spike_threshold: 3.0,
            behavior_change_threshold: 0.7,
            min_baseline_samples: 100,
            anomaly_retention_hours: 24,
            enable_traffic_detection: true,
            enable_security_detection: true,
            enable_behavior_detection: true,
            enable_protocol_detection: true,
        }
    }
}

impl Default for AnomalyDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple hash function for packet fingerprinting
fn simple_hash(data: &[u8]) -> u64 {
    let mut hash: u64 = 0;
    for (i, &byte) in data.iter().enumerate() {
        hash = hash.wrapping_add((byte as u64).wrapping_mul((i as u64).wrapping_add(1)));
    }
    hash
}
