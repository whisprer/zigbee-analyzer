use zigbee_core::{ParsedPacket, MacAddress, FrameType};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAnalyzer {
    pub config: SecurityConfig,
    pub threats: Vec<SecurityThreat>,
    pub devices: HashMap<MacAddress, DeviceSecurity>,
    pub incidents: VecDeque<SecurityIncident>,
    max_incidents: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub monitor_replay_attacks: bool,
    pub monitor_jamming: bool,
    pub monitor_key_transport: bool,
    pub monitor_unauthorized_devices: bool,
    pub replay_window_ms: u64,
    pub jamming_threshold: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceSecurity {
    pub address: MacAddress,
    pub is_encrypted: bool,
    pub encryption_failures: u64,
    pub frame_counters: HashMap<u8, u32>,
    pub last_key_update: Option<SystemTime>,
    pub trust_score: f32,
    pub suspicious_activity_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityThreat {
    pub threat_type: ThreatType,
    pub severity: ThreatSeverity,
    pub source: Option<MacAddress>,
    pub destination: Option<MacAddress>,
    pub description: String,
    pub timestamp: SystemTime,
    pub details: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThreatType {
    ReplayAttack,
    Jamming,
    UnauthorizedDevice,
    WeakEncryption,
    KeyTransport,
    AnomalousTraffic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThreatSeverity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIncident {
    pub incident_type: String,
    pub timestamp: SystemTime,
    pub details: String,
}

impl SecurityAnalyzer {
    pub fn new() -> Self {
        Self {
            config: SecurityConfig::default(),
            threats: Vec::new(),
            devices: HashMap::new(),
            incidents: VecDeque::new(),
            max_incidents: 1000,
        }
    }
    
    pub fn with_config(config: SecurityConfig) -> Self {
        Self {
            config,
            threats: Vec::new(),
            devices: HashMap::new(),
            incidents: VecDeque::new(),
            max_incidents: 1000,
        }
    }
    
    pub fn process_packet(&mut self, parsed: &ParsedPacket) {
        let now = SystemTime::now();
        
        if let Some(src) = parsed.mac.src_addr.as_mac() {
            self.update_device_security(&src, parsed, now);
        }
        
        if parsed.mac.frame_control.frame_type == FrameType::Data {
            if let Some(_nwk) = &parsed.network {
                self.analyze_network_security_wrapper(parsed, now);
            }
            
            if let Some(_aps) = &parsed.aps {
                self.analyze_aps_security_wrapper(parsed, now);
            }
        }
    }
    
    fn update_device_security(&mut self, addr: &MacAddress, parsed: &ParsedPacket, now: SystemTime) {
        // FIX: Access security through frame_control, not directly
        let is_encrypted = parsed.network
            .as_ref()
            .map(|nwk| nwk.frame_control.security)
            .unwrap_or(false);
        
        let dev_sec = self.devices.entry(*addr).or_insert_with(|| DeviceSecurity {
            address: *addr,
            is_encrypted,
            encryption_failures: 0,
            frame_counters: HashMap::new(),
            last_key_update: None,
            trust_score: 50.0,
            suspicious_activity_count: 0,
        });
        
        dev_sec.is_encrypted = is_encrypted;
        
        // Calculate and update trust score
        let trust_score = Self::calculate_trust_score_static(
            dev_sec.is_encrypted,
            dev_sec.encryption_failures,
            dev_sec.suspicious_activity_count,
        );
        dev_sec.trust_score = trust_score;
    }
    
    fn calculate_trust_score_static(
        is_encrypted: bool,
        encryption_failures: u64,
        suspicious_activity_count: u64,
    ) -> f32 {
        let mut score = 50.0;
        
        if is_encrypted {
            score += 30.0;
        }
        
        score -= (encryption_failures as f32) * 5.0;
        score -= (suspicious_activity_count as f32) * 10.0;
        
        score.max(0.0).min(100.0)
    }
    
    fn analyze_network_security_wrapper(&mut self, parsed: &ParsedPacket, now: SystemTime) {
        if let Some(nwk) = &parsed.network {
            // FIX: Access security through frame_control
            if !nwk.frame_control.security {
                if let Some(src) = parsed.mac.src_addr.as_mac() {
                    self.threats.push(SecurityThreat {
                        threat_type: ThreatType::WeakEncryption,
                        severity: ThreatSeverity::High,
                        source: Some(src),
                        destination: parsed.mac.dest_addr.as_mac(),
                        description: "Unencrypted network frame detected".to_string(),
                        timestamp: now,
                        details: HashMap::new(),
                    });
                }
            }
        }
    }
    
    fn analyze_aps_security_wrapper(&mut self, parsed: &ParsedPacket, now: SystemTime) {
        if let Some(aps) = &parsed.aps {
            // FIX: Access security through frame_control
            if !aps.frame_control.security {
                if let Some(src) = parsed.mac.src_addr.as_mac() {
                    self.threats.push(SecurityThreat {
                        threat_type: ThreatType::WeakEncryption,
                        severity: ThreatSeverity::Medium,
                        source: Some(src),
                        destination: parsed.mac.dest_addr.as_mac(),
                        description: "Unencrypted APS frame detected".to_string(),
                        timestamp: now,
                        details: HashMap::new(),
                    });
                }
            }
        }
    }
    
    fn add_incident(&mut self, incident_type: String, details: String) {
        self.incidents.push_back(SecurityIncident {
            incident_type,
            timestamp: SystemTime::now(),
            details,
        });
        
        while self.incidents.len() > self.max_incidents {
            self.incidents.pop_front();
        }
    }
    
    pub fn get_statistics(&self) -> SecurityStatistics {
        SecurityStatistics {
            total_threats: self.threats.len(),
            critical_threats: self.threats.iter()
                .filter(|t| t.severity == ThreatSeverity::Critical)
                .count(),
            high_threats: self.threats.iter()
                .filter(|t| t.severity == ThreatSeverity::High)
                .count(),
            medium_threats: self.threats.iter()
                .filter(|t| t.severity == ThreatSeverity::Medium)
                .count(),
            low_threats: self.threats.iter()
                .filter(|t| t.severity == ThreatSeverity::Low)
                .count(),
            encrypted_devices: self.devices.values()
                .filter(|d| d.is_encrypted)
                .count(),
            unencrypted_devices: self.devices.values()
                .filter(|d| !d.is_encrypted)
                .count(),
        }
    }
    
    pub fn get_threats_by_severity(&self, severity: ThreatSeverity) -> Vec<&SecurityThreat> {
        self.threats.iter()
            .filter(|t| t.severity == severity)
            .collect()
    }
    
    pub fn get_device_security(&self, addr: &MacAddress) -> Option<&DeviceSecurity> {
        self.devices.get(addr)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityStatistics {
    pub total_threats: usize,
    pub critical_threats: usize,
    pub high_threats: usize,
    pub medium_threats: usize,
    pub low_threats: usize,
    pub encrypted_devices: usize,
    pub unencrypted_devices: usize,
}

impl Default for SecurityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            monitor_replay_attacks: true,
            monitor_jamming: true,
            monitor_key_transport: true,
            monitor_unauthorized_devices: true,
            replay_window_ms: 1000,
            jamming_threshold: 100,
        }
    }
}
