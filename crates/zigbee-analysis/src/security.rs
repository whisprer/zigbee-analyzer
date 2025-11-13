use zigbee_core::{ParsedPacket, MacAddress};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{SystemTime, Duration};

/// Security analyzer for Zigbee networks
pub struct SecurityAnalyzer {
    // Network security state
    networks: HashMap<u16, NetworkSecurity>,
    
    // Device security profiles
    devices: HashMap<MacAddress, DeviceSecurity>,
    
    // Key material tracking
    key_events: Vec<KeyEvent>,
    
    // Security incidents
    incidents: Vec<SecurityIncident>,
    
    // Join/pairing monitoring
    join_attempts: VecDeque<JoinAttempt>,
    
    // Attack detection
    attack_patterns: HashMap<AttackType, u32>,
    
    // Configuration
    config: SecurityConfig,
}

#[derive(Debug, Clone)]
pub struct SecurityConfig {
    pub track_unencrypted_traffic: bool,
    pub detect_downgrade_attacks: bool,
    pub monitor_key_transport: bool,
    pub log_join_attempts: bool,
    pub alert_on_promiscuous: bool,
}

/// Security state for a network (PAN)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSecurity {
    pub pan_id: u16,
    pub security_enabled: bool,
    pub trust_center: Option<MacAddress>,
    pub security_level: SecurityLevel,
    
    // Key information
    pub network_key_present: bool,
    pub link_keys_count: usize,
    pub key_sequence_number: Option<u8>,
    
    // Traffic analysis
    pub encrypted_packets: u64,
    pub unencrypted_packets: u64,
    pub encryption_rate: f32,
    
    // Device counts
    pub secured_devices: usize,
    pub unsecured_devices: usize,
    
    // Vulnerabilities
    pub vulnerabilities: Vec<Vulnerability>,
    
    // Last update
    pub last_seen: SystemTime,
    pub first_seen: SystemTime,
}

/// Security profile for a device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceSecurity {
    pub mac_addr: MacAddress,
    pub nwk_addr: Option<u16>,
    
    // Security capabilities
    pub supports_encryption: bool,
    pub encryption_enabled: bool,
    pub security_level: SecurityLevel,
    
    // Key status
    pub has_network_key: bool,
    pub has_link_key: bool,
    pub key_type: Option<KeyType>,
    
    // Behavior
    pub tx_encrypted: u64,
    pub tx_unencrypted: u64,
    pub rx_encrypted: u64,
    pub rx_unencrypted: u64,
    
    // Trust
    pub is_trusted: bool,
    pub trust_score: f32, // 0.0 - 1.0
    
    // Flags
    pub is_trust_center: bool,
    pub is_coordinator: bool,
    
    // Security events
    pub failed_auth_attempts: u32,
    pub key_updates: u32,
    
    // Timestamps
    pub first_seen: SystemTime,
    pub last_seen: SystemTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityLevel {
    None,           // No security
    Mic32,          // 32-bit MIC
    Mic64,          // 64-bit MIC
    Mic128,         // 128-bit MIC
    Encrypted,      // Encryption only
    EncMic32,       // Encryption + 32-bit MIC
    EncMic64,       // Encryption + 64-bit MIC
    EncMic128,      // Encryption + 128-bit MIC (standard)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyType {
    NetworkKey,
    LinkKey,
    TrustCenterLinkKey,
    Unknown,
}

/// Key-related event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyEvent {
    pub timestamp: SystemTime,
    pub event_type: KeyEventType,
    pub device: MacAddress,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyEventType {
    KeyTransport,
    KeyUpdate,
    KeyRequest,
    KeyEstablishment,
    KeyVerification,
}

/// Security incident
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIncident {
    pub id: u64,
    pub timestamp: SystemTime,
    pub incident_type: IncidentType,
    pub severity: IncidentSeverity,
    pub description: String,
    pub affected_device: Option<MacAddress>,
    pub evidence: Vec<String>,
    pub mitigations: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IncidentType {
    UnencryptedTraffic,
    WeakSecurity,
    MissingEncryption,
    KeyTransportInClear,
    DowngradeAttempt,
    UnauthorizedAccess,
    PromiscuousMode,
    InsecureJoin,
    ReplayAttack,
    KeyCompromise,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum IncidentSeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

/// Join/pairing attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinAttempt {
    pub timestamp: SystemTime,
    pub device: MacAddress,
    pub success: bool,
    pub method: JoinMethod,
    pub security_level: SecurityLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JoinMethod {
    Association,
    Rejoin,
    DirectJoin,
    Unknown,
}

/// Vulnerability in the network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    pub vuln_type: VulnerabilityType,
    pub severity: IncidentSeverity,
    pub description: String,
    pub affected_devices: Vec<MacAddress>,
    pub recommendation: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VulnerabilityType {
    NoEncryption,
    WeakEncryption,
    DefaultKeys,
    InsecureKeyTransport,
    MissingAccessControl,
    OpenNetwork,
    LegacyProtocol,
    UnpatchedDevice,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AttackType {
    Eavesdropping,
    Jamming,
    Replay,
    ManInTheMiddle,
    KeyExtraction,
    Impersonation,
    DenialOfService,
}

impl SecurityAnalyzer {
    pub fn new() -> Self {
        Self::with_config(SecurityConfig::default())
    }
    
    pub fn with_config(config: SecurityConfig) -> Self {
        Self {
            networks: HashMap::new(),
            devices: HashMap::new(),
            key_events: Vec::new(),
            incidents: Vec::new(),
            join_attempts: VecDeque::new(),
            attack_patterns: HashMap::new(),
            config,
        }
    }
    
    /// Process a parsed packet for security analysis
    pub fn process_packet(&mut self, parsed: &ParsedPacket) {
        let now = SystemTime::now();
        
        // Analyze MAC layer security
        self.analyze_mac_security(parsed, now);
        
        // Analyze network layer security
        if let Some(nwk) = &parsed.network {
            self.analyze_network_security(parsed, nwk, now);
        }
        
        // Analyze APS layer security
        if let Some(aps) = &parsed.aps {
            self.analyze_aps_security(parsed, aps, now);
        }
        
        // Update network security state
        if let Some(pan_id) = parsed.mac.src_pan.or(parsed.mac.dest_pan) {
            self.update_network_security(pan_id, parsed, now);
        }
        
        // Update device security profiles
        self.update_device_security(&parsed.mac.src_addr, parsed, true, now);
        if !parsed.mac.dest_addr.is_broadcast() && !parsed.mac.dest_addr.is_none() {
            self.update_device_security(&parsed.mac.dest_addr, parsed, false, now);
        }
    }
    
    fn analyze_mac_security(&mut self, parsed: &ParsedPacket, now: SystemTime) {
        let security_enabled = parsed.mac.frame_control.security_enabled;
        
        // Check for unencrypted data frames
        if !security_enabled 
            && parsed.mac.frame_control.frame_type == zigbee_core::FrameType::Data
            && self.config.track_unencrypted_traffic {
            
            self.record_incident(SecurityIncident {
                id: self.incidents.len() as u64,
                timestamp: now,
                incident_type: IncidentType::UnencryptedTraffic,
                severity: IncidentSeverity::Medium,
                description: format!("Unencrypted data frame from {}", parsed.mac.src_addr),
                affected_device: Some(parsed.mac.src_addr),
                evidence: vec![
                    "MAC layer security not enabled".to_string(),
                    format!("Frame type: {:?}", parsed.mac.frame_control.frame_type),
                ],
                mitigations: vec![
                    "Enable network-wide encryption".to_string(),
                    "Configure security on all devices".to_string(),
                ],
            });
        }
    }
    
    fn analyze_network_security(&mut self, parsed: &ParsedPacket, nwk: &zigbee_core::NetworkFrame, now: SystemTime) {
        // Check for network layer security
        if nwk.frame_control.security {
            // Good - security enabled
        } else if nwk.frame_control.frame_type == zigbee_core::NwkFrameType::Data {
            // Network data without security - potential issue
            self.record_incident(SecurityIncident {
                id: self.incidents.len() as u64,
                timestamp: now,
                incident_type: IncidentType::MissingEncryption,
                severity: IncidentSeverity::High,
                description: format!("Network data frame without security from 0x{:04x}", nwk.src_addr),
                affected_device: Some(parsed.mac.src_addr),
                evidence: vec![
                    "Network layer security not enabled".to_string(),
                    format!("NWK src: 0x{:04x}", nwk.src_addr),
                ],
                mitigations: vec![
                    "Enable network layer security".to_string(),
                    "Review device security configuration".to_string(),
                ],
            });
        }
    }
    
    fn analyze_aps_security(&mut self, parsed: &ParsedPacket, aps: &zigbee_core::ApsFrame, now: SystemTime) {
        // Check for APS layer security
        if aps.frame_control.security {
            // Security enabled at APS layer
        } else if aps.frame_control.frame_type == zigbee_core::ApsFrameType::Data {
            // Data without APS security
            self.record_incident(SecurityIncident {
                id: self.incidents.len() as u64,
                timestamp: now,
                incident_type: IncidentType::WeakSecurity,
                severity: IncidentSeverity::Low,
                description: "APS data frame without security".to_string(),
                affected_device: Some(parsed.mac.src_addr),
                evidence: vec!["APS layer security not enabled".to_string()],
                mitigations: vec!["Consider enabling APS layer security".to_string()],
            });
        }
        
        // Check for key transport
        if self.config.monitor_key_transport && aps.frame_control.frame_type == zigbee_core::ApsFrameType::Command {
            // This might be key transport - flag it
            self.key_events.push(KeyEvent {
                timestamp: now,
                event_type: KeyEventType::KeyTransport,
                device: parsed.mac.src_addr,
                description: "Potential key transport detected".to_string(),
            });
        }
    }
    
    fn update_network_security(&mut self, pan_id: u16, parsed: &ParsedPacket, now: SystemTime) {
        let net_sec = self.networks.entry(pan_id).or_insert_with(|| NetworkSecurity {
            pan_id,
            security_enabled: false,
            trust_center: None,
            security_level: SecurityLevel::None,
            network_key_present: false,
            link_keys_count: 0,
            key_sequence_number: None,
            encrypted_packets: 0,
            unencrypted_packets: 0,
            encryption_rate: 0.0,
            secured_devices: 0,
            unsecured_devices: 0,
            vulnerabilities: Vec::new(),
            last_seen: now,
            first_seen: now,
        });
        
        net_sec.last_seen = now;
        
        // Update encryption counters
        if parsed.mac.frame_control.security_enabled {
            net_sec.encrypted_packets += 1;
            net_sec.security_enabled = true;
        } else {
            net_sec.unencrypted_packets += 1;
        }
        
        // Calculate encryption rate
        let total = net_sec.encrypted_packets + net_sec.unencrypted_packets;
        if total > 0 {
            net_sec.encryption_rate = net_sec.encrypted_packets as f32 / total as f32;
        }
        
        // Detect trust center (coordinator at 0x0000)
        if let Some(nwk) = &parsed.network {
            if nwk.src_addr == 0x0000 {
                net_sec.trust_center = Some(parsed.mac.src_addr);
            }
        }
    }
    
    fn update_device_security(&mut self, addr: &MacAddress, parsed: &ParsedPacket, is_tx: bool, now: SystemTime) {
        if addr.is_none() || addr.is_broadcast() {
            return;
        }
        
        let dev_sec = self.devices.entry(*addr).or_insert_with(|| DeviceSecurity {
            mac_addr: *addr,
            nwk_addr: None,
            supports_encryption: false,
            encryption_enabled: false,
            security_level: SecurityLevel::None,
            has_network_key: false,
            has_link_key: false,
            key_type: None,
            tx_encrypted: 0,
            tx_unencrypted: 0,
            rx_encrypted: 0,
            rx_unencrypted: 0,
            is_trusted: false,
            trust_score: 0.5,
            is_trust_center: false,
            is_coordinator: false,
            failed_auth_attempts: 0,
            key_updates: 0,
            first_seen: now,
            last_seen: now,
        });
        
        dev_sec.last_seen = now;
        
        // Update network address
        if let Some(nwk) = &parsed.network {
            if is_tx {
                dev_sec.nwk_addr = Some(nwk.src_addr);
                if nwk.src_addr == 0x0000 {
                    dev_sec.is_coordinator = true;
                    dev_sec.is_trust_center = true;
                }
            }
        }
        
        // Update encryption counters
        let encrypted = parsed.mac.frame_control.security_enabled;
        
        if is_tx {
            if encrypted {
                dev_sec.tx_encrypted += 1;
                dev_sec.encryption_enabled = true;
                dev_sec.supports_encryption = true;
            } else {
                dev_sec.tx_unencrypted += 1;
            }
        } else {
            if encrypted {
                dev_sec.rx_encrypted += 1;
            } else {
                dev_sec.rx_unencrypted += 1;
            }
        }
        
        // Calculate trust score
        self.calculate_trust_score(dev_sec);
    }
    
    fn calculate_trust_score(&self, device: &mut DeviceSecurity) {
        let mut score = 0.5f32;
        
        // Encryption usage (0-30 points)
        let total_tx = device.tx_encrypted + device.tx_unencrypted;
        if total_tx > 0 {
            let encryption_rate = device.tx_encrypted as f32 / total_tx as f32;
            score += encryption_rate * 0.3;
        }
        
        // Is trusted/known device (0-20 points)
        if device.is_trusted {
            score += 0.2;
        }
        
        // Trust center/coordinator (0-20 points)
        if device.is_trust_center || device.is_coordinator {
            score += 0.2;
        }
        
        // No failed auth attempts (0-15 points)
        if device.failed_auth_attempts == 0 {
            score += 0.15;
        } else {
            score -= (device.failed_auth_attempts as f32 * 0.05).min(0.15);
        }
        
        // Has security keys (0-15 points)
        if device.has_network_key {
            score += 0.075;
        }
        if device.has_link_key {
            score += 0.075;
        }
        
        device.trust_score = score.clamp(0.0, 1.0);
    }
    
    fn record_incident(&mut self, incident: SecurityIncident) {
        self.incidents.push(incident);
        
        // Keep only last 1000 incidents
        if self.incidents.len() > 1000 {
            self.incidents.remove(0);
        }
    }
    
    /// Record a join attempt
    pub fn record_join_attempt(&mut self, device: MacAddress, success: bool, method: JoinMethod, security_level: SecurityLevel) {
        let attempt = JoinAttempt {
            timestamp: SystemTime::now(),
            device,
            success,
            method,
            security_level,
        };
        
        self.join_attempts.push_back(attempt);
        
        // Keep only last 500 attempts
        if self.join_attempts.len() > 500 {
            self.join_attempts.pop_front();
        }
        
        // Check for insecure join
        if success && security_level == SecurityLevel::None {
            self.record_incident(SecurityIncident {
                id: self.incidents.len() as u64,
                timestamp: SystemTime::now(),
                incident_type: IncidentType::InsecureJoin,
                severity: IncidentSeverity::High,
                description: format!("Insecure join (no encryption) by {}", device),
                affected_device: Some(device),
                evidence: vec![
                    "Device joined without security".to_string(),
                    format!("Join method: {:?}", method),
                ],
                mitigations: vec![
                    "Require security for all joins".to_string(),
                    "Use install codes or link keys".to_string(),
                ],
            });
        }
    }
    
    /// Get network security assessment
    pub fn get_network_assessment(&self, pan_id: u16) -> Option<NetworkAssessment> {
        let net_sec = self.networks.get(&pan_id)?;
        
        let mut assessment = NetworkAssessment {
            pan_id,
            overall_score: 0.0,
            security_level: net_sec.security_level,
            encryption_rate: net_sec.encryption_rate,
            issues: Vec::new(),
            recommendations: Vec::new(),
        };
        
        // Calculate overall score
        let mut score = 0.0;
        
        // Encryption rate (0-40 points)
        score += net_sec.encryption_rate * 40.0;
        
        // Security enabled (0-30 points)
        if net_sec.security_enabled {
            score += 30.0;
        }
        
        // Trust center present (0-15 points)
        if net_sec.trust_center.is_some() {
            score += 15.0;
        }
        
        // Network key present (0-15 points)
        if net_sec.network_key_present {
            score += 15.0;
        }
        
        assessment.overall_score = score;
        
        // Identify issues
        if net_sec.encryption_rate < 0.95 {
            assessment.issues.push("Significant unencrypted traffic detected".to_string());
            assessment.recommendations.push("Enable encryption on all devices".to_string());
        }
        
        if !net_sec.security_enabled {
            assessment.issues.push("Network security not enabled".to_string());
            assessment.recommendations.push("Enable network-wide security".to_string());
        }
        
        if net_sec.trust_center.is_none() {
            assessment.issues.push("No trust center identified".to_string());
            assessment.recommendations.push("Configure trust center".to_string());
        }
        
        if net_sec.security_level == SecurityLevel::None {
            assessment.issues.push("No security level configured".to_string());
            assessment.recommendations.push("Configure security level 5 (EncMic128)".to_string());
        }
        
        Some(assessment)
    }
    
    /// Get device security assessment
    pub fn get_device_assessment(&self, device: &MacAddress) -> Option<DeviceAssessment> {
        let dev_sec = self.devices.get(device)?;
        
        let mut assessment = DeviceAssessment {
            mac_addr: *device,
            trust_score: dev_sec.trust_score,
            encryption_enabled: dev_sec.encryption_enabled,
            security_level: dev_sec.security_level,
            issues: Vec::new(),
            recommendations: Vec::new(),
        };
        
        // Identify issues
        let total_tx = dev_sec.tx_encrypted + dev_sec.tx_unencrypted;
        if total_tx > 0 {
            let enc_rate = dev_sec.tx_encrypted as f32 / total_tx as f32;
            if enc_rate < 0.9 {
                assessment.issues.push(format!("Low encryption rate: {:.1}%", enc_rate * 100.0));
                assessment.recommendations.push("Enable encryption for all transmissions".to_string());
            }
        }
        
        if !dev_sec.encryption_enabled {
            assessment.issues.push("Encryption not enabled".to_string());
            assessment.recommendations.push("Enable device encryption".to_string());
        }
        
        if !dev_sec.has_network_key {
            assessment.issues.push("No network key".to_string());
            assessment.recommendations.push("Provision network key".to_string());
        }
        
        if dev_sec.failed_auth_attempts > 0 {
            assessment.issues.push(format!("{} failed authentication attempts", dev_sec.failed_auth_attempts));
            assessment.recommendations.push("Investigate authentication failures".to_string());
        }
        
        Some(assessment)
    }
    
    /// Get all security incidents
    pub fn get_incidents(&self) -> &[SecurityIncident] {
        &self.incidents
    }
    
    /// Get incidents by type
    pub fn get_incidents_by_type(&self, incident_type: IncidentType) -> Vec<&SecurityIncident> {
        self.incidents.iter()
            .filter(|i| i.incident_type == incident_type)
            .collect()
    }
    
    /// Get incidents by severity
    pub fn get_incidents_by_severity(&self, min_severity: IncidentSeverity) -> Vec<&SecurityIncident> {
        self.incidents.iter()
            .filter(|i| i.severity >= min_severity)
            .collect()
    }
    
    /// Get join attempts
    pub fn get_join_attempts(&self) -> Vec<&JoinAttempt> {
        self.join_attempts.iter().collect()
    }
    
    /// Get key events
    pub fn get_key_events(&self) -> &[KeyEvent] {
        &self.key_events
    }
    
    /// Get security statistics
    pub fn get_statistics(&self) -> SecurityStatistics {
        let mut stats = SecurityStatistics {
            total_incidents: self.incidents.len(),
            critical_incidents: 0,
            high_incidents: 0,
            medium_incidents: 0,
            low_incidents: 0,
            info_incidents: 0,
            networks_analyzed: self.networks.len(),
            devices_analyzed: self.devices.len(),
            secured_devices: 0,
            unsecured_devices: 0,
            avg_encryption_rate: 0.0,
            avg_trust_score: 0.0,
            join_attempts: self.join_attempts.len(),
            successful_joins: 0,
            failed_joins: 0,
            key_events: self.key_events.len(),
        };
        
        for incident in &self.incidents {
            match incident.severity {
                IncidentSeverity::Critical => stats.critical_incidents += 1,
                IncidentSeverity::High => stats.high_incidents += 1,
                IncidentSeverity::Medium => stats.medium_incidents += 1,
                IncidentSeverity::Low => stats.low_incidents += 1,
                IncidentSeverity::Info => stats.info_incidents += 1,
            }
        }
        
        for device in self.devices.values() {
            if device.encryption_enabled {
                stats.secured_devices += 1;
            } else {
                stats.unsecured_devices += 1;
            }
            stats.avg_trust_score += device.trust_score;
        }
        
        if !self.devices.is_empty() {
            stats.avg_trust_score /= self.devices.len() as f32;
        }
        
        let mut total_enc_rate = 0.0;
        for network in self.networks.values() {
            total_enc_rate += network.encryption_rate;
        }
        if !self.networks.is_empty() {
            stats.avg_encryption_rate = total_enc_rate / self.networks.len() as f32;
        }
        
        for attempt in &self.join_attempts {
            if attempt.success {
                stats.successful_joins += 1;
            } else {
                stats.failed_joins += 1;
            }
        }
        
        stats
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkAssessment {
    pub pan_id: u16,
    pub overall_score: f32,
    pub security_level: SecurityLevel,
    pub encryption_rate: f32,
    pub issues: Vec<String>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceAssessment {
    pub mac_addr: MacAddress,
    pub trust_score: f32,
    pub encryption_enabled: bool,
    pub security_level: SecurityLevel,
    pub issues: Vec<String>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityStatistics {
    pub total_incidents: usize,
    pub critical_incidents: usize,
    pub high_incidents: usize,
    pub medium_incidents: usize,
    pub low_incidents: usize,
    pub info_incidents: usize,
    pub networks_analyzed: usize,
    pub devices_analyzed: usize,
    pub secured_devices: usize,
    pub unsecured_devices: usize,
    pub avg_encryption_rate: f32,
    pub avg_trust_score: f32,
    pub join_attempts: usize,
    pub successful_joins: usize,
    pub failed_joins: usize,
    pub key_events: usize,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            track_unencrypted_traffic: true,
            detect_downgrade_attacks: true,
            monitor_key_transport: true,
            log_join_attempts: true,
            alert_on_promiscuous: true,
        }
    }
}

impl Default for SecurityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}