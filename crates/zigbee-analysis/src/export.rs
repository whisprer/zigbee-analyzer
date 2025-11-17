use crate::{
    TopologyMap, TrafficStatistics, ChannelAnalyzer, AnomalyDetector, 
    SecurityAnalyzer, DeviceDatabase,
};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::time::SystemTime;

/// Export manager for all analysis data
pub struct ExportManager {
    // Export configuration
    config: ExportConfig,
}

#[derive(Debug, Clone)]
pub struct ExportConfig {
    pub include_timestamps: bool,
    pub pretty_print_json: bool,
    pub csv_delimiter: char,
    pub include_raw_data: bool,
}

/// Supported export formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    JSON,
    CSV,
    HTML,
    Markdown,
    XML,
    PDF,
}

/// Complete analysis snapshot for export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisSnapshot {
    pub metadata: SnapshotMetadata,
    pub topology: Option<TopologyExport>,
    pub statistics: Option<StatisticsExport>,
    pub channels: Option<ChannelExport>,
    pub anomalies: Option<AnomalyExport>,
    pub security: Option<SecurityExport>,
    pub devices: Option<DeviceExport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    pub timestamp: SystemTime,
    pub duration_seconds: u64,
    pub total_packets: u64,
    pub analyzer_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyExport {
    pub devices: Vec<DeviceExportEntry>,
    pub links: Vec<LinkExportEntry>,
    pub networks: Vec<NetworkExportEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceExportEntry {
    pub mac_addr: String,
    pub nwk_addr: Option<String>,
    pub device_type: String,
    pub pan_id: Option<String>,
    pub packet_count: usize,
    pub avg_rssi: f32,
    pub avg_lqi: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkExportEntry {
    pub source: String,
    pub destination: String,
    pub packet_count: usize,
    pub avg_rssi: f32,
    pub avg_lqi: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkExportEntry {
    pub pan_id: String,
    pub device_count: usize,
    pub coordinator: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticsExport {
    pub total_packets: u64,
    pub total_bytes: u64,
    pub avg_packet_size: f32,
    pub packets_per_second: f32,
    pub frame_types: FrameTypeCounts,
    pub unique_devices: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameTypeCounts {
    pub beacon: u64,
    pub data: u64,
    pub ack: u64,
    pub command: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelExport {
    pub channels: Vec<ChannelExportEntry>,
    pub recommended_channel: u8,
    pub interference_detected: Vec<ChannelInterference>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelExportEntry {
    pub channel: u8,
    pub frequency_mhz: u16,
    pub quality_score: f32,
    pub packet_count: u64,
    pub avg_rssi: f32,
    pub utilization: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelInterference {
    pub channel: u8,
    pub interference_type: String,
    pub severity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyExport {
    pub total_anomalies: usize,
    pub by_severity: SeverityCounts,
    pub anomalies: Vec<AnomalyExportEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeverityCounts {
    pub critical: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyExportEntry {
    pub timestamp: SystemTime,
    pub anomaly_type: String,
    pub severity: String,
    pub description: String,
    pub affected_device: Option<String>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityExport {
    pub total_incidents: usize,
    pub by_severity: SeverityCounts,
    pub avg_encryption_rate: f32,
    pub avg_trust_score: f32,
    pub incidents: Vec<SecurityIncidentEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIncidentEntry {
    pub timestamp: SystemTime,
    pub incident_type: String,
    pub severity: String,
    pub description: String,
    pub affected_device: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceExport {
    pub total_devices: usize,
    pub identified: usize,
    pub unidentified: usize,
    pub devices: Vec<DeviceFingerprintEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceFingerprintEntry {
    pub mac_addr: String,
    pub device_type: String,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub confidence: f32,
    pub packet_count: u64,
}

impl ExportManager {
    pub fn new() -> Self {
        Self::with_config(ExportConfig::default())
    }
    
    pub fn with_config(config: ExportConfig) -> Self {
        Self { config }
    }
    
    /// Create a complete analysis snapshot
    pub fn create_snapshot(
        &self,
        topology: Option<&TopologyMap>,
        statistics: Option<&TrafficStatistics>,
        channels: Option<&ChannelAnalyzer>,
        anomalies: Option<&AnomalyDetector>,
        security: Option<&SecurityAnalyzer>,
        devices: Option<&DeviceDatabase>,
        start_time: SystemTime,
    ) -> AnalysisSnapshot {
        let duration = SystemTime::now()
            .duration_since(start_time)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        
        let total_packets = statistics
            .map(|s| s.total_packets)
            .unwrap_or(0);
        
        AnalysisSnapshot {
            metadata: SnapshotMetadata {
                timestamp: SystemTime::now(),
                duration_seconds: duration,
                total_packets,
                analyzer_version: "0.1.0".to_string(),
            },
            topology: topology.map(|t| self.export_topology(t)),
            statistics: statistics.map(|s| self.export_statistics(s)),
            channels: channels.map(|c| self.export_channels(c)),
            anomalies: anomalies.map(|a| self.export_anomalies(a)),
            security: security.map(|s| self.export_security(s)),
            devices: devices.map(|d| self.export_devices(d)),
        }
    }
    
    fn export_topology(&self, topology: &TopologyMap) -> TopologyExport {
        let devices = topology.devices.values()
            .map(|d| DeviceExportEntry {
                mac_addr: format!("{}", d.address),
                nwk_addr: d.short_addr.map(|a| format!("0x{:04x}", a)),
                device_type: format!("{:?}", d.device_type),
                pan_id: d.pan_id.map(|p| format!("0x{:04x}", p)),
                packet_count: 0,  // TODO: Track in Device struct
                avg_rssi: 0.0,  // TODO: Track in Device struct
                avg_lqi: 0.0,  // TODO: Track in Device struct
            })
            .collect();
        
        let links = topology.links.iter()
            .map(|l| LinkExportEntry {
                source: format!("{}", l.source),
                destination: format!("{}", l.destination),
                packet_count: l.packet_count as usize,
                avg_rssi: l.rssi as f32,
                avg_lqi: l.link_quality as f32,
            })
            .collect();
        
        let networks = topology.networks.values()
            .map(|n| NetworkExportEntry {
                pan_id: format!("0x{:04x}", n.pan_id),
                device_count: n.device_count,
                coordinator: n.coordinator.map(|c| format!("{}", c)),
            })
            .collect();
        
        TopologyExport {
            devices,
            links,
            networks,
        }
    }
    
    fn export_statistics(&self, stats: &TrafficStatistics) -> StatisticsExport {
        let summary = stats.get_summary();
        
        StatisticsExport {
            total_packets: summary.total_packets,
            total_bytes: summary.total_bytes,
            avg_packet_size: summary.avg_packet_size,
            packets_per_second: summary.packets_per_second,
            frame_types: FrameTypeCounts {
                beacon: summary.beacon_frames,
                data: summary.data_frames,
                ack: summary.ack_frames,
                command: summary.command_frames,
            },
            unique_devices: summary.unique_devices,
        }
    }
    
    fn export_channels(&self, channels: &ChannelAnalyzer) -> ChannelExport {
        let channels_data = channels.get_all_channels().values()
            .map(|c| ChannelExportEntry {
                channel: c.channel,
                frequency_mhz: c.frequency_mhz,
                quality_score: c.quality_score,
                packet_count: c.packet_count,
                avg_rssi: c.avg_rssi,
                utilization: c.utilization,
            })
            .collect();
        
        let recommendation = channels.recommend_channel();
        
        let interference = channels.get_channels_with_interference(0.3)
            .into_iter()
            .map(|(ch, itype, severity)| ChannelInterference {
                channel: ch,
                interference_type: format!("{:?}", itype),
                severity,
            })
            .collect();
        
        ChannelExport {
            channels: channels_data,
            recommended_channel: recommendation.recommended_channel,
            interference_detected: interference,
        }
    }
    
    fn export_anomalies(&self, anomalies: &AnomalyDetector) -> AnomalyExport {
        let stats = anomalies.get_statistics();
        
        let anomaly_list = anomalies.get_anomalies()
            .iter()
            .map(|a| AnomalyExportEntry {
                timestamp: a.timestamp,
                anomaly_type: format!("{:?}", a.anomaly_type),
                severity: format!("{:?}", a.severity),
                description: a.description.clone(),
                affected_device: a.affected_device.map(|d| format!("{}", d)),
                confidence: a.confidence,
            })
            .collect();
        
        AnomalyExport {
            total_anomalies: stats.total_anomalies,
            by_severity: SeverityCounts {
                critical: stats.critical,
                high: stats.high,
                medium: stats.medium,
                low: stats.low,
            },
            anomalies: anomaly_list,
        }
    }
    
    fn export_security(&self, security: &SecurityAnalyzer) -> SecurityExport {
        let stats = security.get_statistics();
        
        let incident_list: Vec<SecurityIncidentEntry> = security.incidents
            .iter()
            .map(|i| SecurityIncidentEntry {
                timestamp: i.timestamp,
                incident_type: i.incident_type.clone(),
                severity: "Unknown".to_string(),
                description: i.details.clone(),
                affected_device: None,
            })
            .collect();
        
        SecurityExport {
            total_incidents: stats.total_threats,
            by_severity: SeverityCounts {
                critical: stats.critical_threats,
                high: stats.high_threats,
                medium: stats.medium_threats,
                low: stats.low_threats,
            },
            avg_encryption_rate: 0.0,  // TODO: Calculate encryption rate,
            avg_trust_score: 50.0,  // TODO: Calculate trust score,
            incidents: incident_list,
        }
    }
    
    fn export_devices(&self, devices: &DeviceDatabase) -> DeviceExport {
        let stats = devices.get_statistics();
        
        let device_list = devices.devices.values()
            .map(|d| DeviceFingerprintEntry {
                mac_addr: format!("{}", d.address),
                device_type: format!("{:?}", d.device_type),
                manufacturer: d.manufacturer.clone(),
                model: d.model.clone(),
                confidence: 1.0,
                packet_count: d.packet_count,
            })
            .collect();
        
        DeviceExport {
            total_devices: stats.total_devices,
            identified: stats.total_devices,  // TODO: Track identified,
            unidentified: 0,  // TODO: Track unidentified,
            devices: device_list,
        }
    }
    
    /// Export snapshot to JSON
    pub fn export_json<P: AsRef<Path>>(&self, snapshot: &AnalysisSnapshot, path: P) -> std::io::Result<()> {
        let json = if self.config.pretty_print_json {
            serde_json::to_string_pretty(snapshot)?
        } else {
            serde_json::to_string(snapshot)?
        };
        
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }
    
    /// Export topology to CSV
    pub fn export_topology_csv<P: AsRef<Path>>(&self, topology: &TopologyMap, path: P) -> std::io::Result<()> {
        let mut file = File::create(path)?;
        let delim = self.config.csv_delimiter;
        
        // Header
        writeln!(file, "MAC Address{}NWK Address{}Device Type{}PAN ID{}Packets{}Avg RSSI{}Avg LQI",
            delim, delim, delim, delim, delim, delim)?;
        
        // Data
        for device in topology.devices.values() {
            writeln!(file, "{}{}{}{}{}{}{}{}{}{}{}{}{}",
                device.address,
                delim,
                device.short_addr.map(|a| format!("0x{:04x}", a)).unwrap_or_default(),
                delim,
                format!("{:?}", device.device_type),
                delim,
                device.pan_id.map(|p| format!("0x{:04x}", p)).unwrap_or_default(),
                delim,
                0,  // TODO
                delim,
                0,  // TODO
                delim,
                0  // TODO
            )?;
        }
        
        Ok(())
    }
    
    /// Export anomalies to CSV
    pub fn export_anomalies_csv<P: AsRef<Path>>(&self, anomalies: &AnomalyDetector, path: P) -> std::io::Result<()> {
        let mut file = File::create(path)?;
        let delim = self.config.csv_delimiter;
        
        // Header
        writeln!(file, "Timestamp{}Type{}Severity{}Description{}Device{}Confidence",
            delim, delim, delim, delim, delim)?;
        
        // Data
        for anomaly in anomalies.get_anomalies() {
            let timestamp = format!("{:?}", anomaly.timestamp);
            let device = anomaly.affected_device.map(|d| format!("{}", d)).unwrap_or_default();
            
            writeln!(file, "{}{}{}{}{}{}{}{}{}{}{}",
                timestamp,
                delim,
                format!("{:?}", anomaly.anomaly_type),
                delim,
                format!("{:?}", anomaly.severity),
                delim,
                anomaly.description.replace(&delim.to_string(), " "),
                delim,
                device,
                delim,
                anomaly.confidence
            )?;
        }
        
        Ok(())
    }
    
    /// Export to HTML report
    pub fn export_html_report<P: AsRef<Path>>(&self, snapshot: &AnalysisSnapshot, path: P) -> std::io::Result<()> {
        let mut html = String::new();
        
        // HTML header
        html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        html.push_str("<meta charset=\"UTF-8\">\n");
        html.push_str("<title>Zigbee Analysis Report</title>\n");
        html.push_str("<style>\n");
        html.push_str(include_str!("../resources/report.css"));
        html.push_str("</style>\n");
        html.push_str("</head>\n<body>\n");
        
        // Title
        html.push_str("<h1>Zigbee Network Analysis Report</h1>\n");
        html.push_str(&format!("<p>Generated: {:?}</p>\n", snapshot.metadata.timestamp));
        html.push_str(&format!("<p>Duration: {} seconds</p>\n", snapshot.metadata.duration_seconds));
        html.push_str(&format!("<p>Total Packets: {}</p>\n", snapshot.metadata.total_packets));
        
        // Statistics section
        if let Some(ref stats) = snapshot.statistics {
            html.push_str("<h2>Traffic Statistics</h2>\n");
            html.push_str("<table>\n");
            html.push_str("<tr><th>Metric</th><th>Value</th></tr>\n");
            html.push_str(&format!("<tr><td>Total Packets</td><td>{}</td></tr>\n", stats.total_packets));
            html.push_str(&format!("<tr><td>Total Bytes</td><td>{}</td></tr>\n", stats.total_bytes));
            html.push_str(&format!("<tr><td>Avg Packet Size</td><td>{:.1} bytes</td></tr>\n", stats.avg_packet_size));
            html.push_str(&format!("<tr><td>Packets/Second</td><td>{:.1}</td></tr>\n", stats.packets_per_second));
            html.push_str(&format!("<tr><td>Unique Devices</td><td>{}</td></tr>\n", stats.unique_devices));
            html.push_str("</table>\n");
        }
        
        // Topology section
        if let Some(ref topo) = snapshot.topology {
            html.push_str("<h2>Network Topology</h2>\n");
            html.push_str(&format!("<p>Total Devices: {}</p>\n", topo.devices.len()));
            html.push_str(&format!("<p>Total Links: {}</p>\n", topo.links.len()));
            html.push_str(&format!("<p>Networks: {}</p>\n", topo.networks.len()));
        }
        
        // Anomalies section
        if let Some(ref anomalies) = snapshot.anomalies {
            html.push_str("<h2>Anomalies</h2>\n");
            html.push_str(&format!("<p>Total: {} (Critical: {}, High: {}, Medium: {}, Low: {})</p>\n",
                anomalies.total_anomalies,
                anomalies.by_severity.critical,
                anomalies.by_severity.high,
                anomalies.by_severity.medium,
                anomalies.by_severity.low
            ));
            
            if !anomalies.anomalies.is_empty() {
                html.push_str("<table>\n");
                html.push_str("<tr><th>Type</th><th>Severity</th><th>Description</th></tr>\n");
                for anomaly in anomalies.anomalies.iter().take(20) {
                    html.push_str(&format!("<tr><td>{}</td><td>{}</td><td>{}</td></tr>\n",
                        anomaly.anomaly_type,
                        anomaly.severity,
                        anomaly.description
                    ));
                }
                html.push_str("</table>\n");
            }
        }
        
        // Security section
        if let Some(ref security) = snapshot.security {
            html.push_str("<h2>Security Analysis</h2>\n");
            html.push_str(&format!("<p>Total Incidents: {}</p>\n", security.total_incidents));
            html.push_str(&format!("<p>Avg Encryption Rate: {:.1}%</p>\n", security.avg_encryption_rate * 100.0));
            html.push_str(&format!("<p>Avg Trust Score: {:.2}/1.0</p>\n", security.avg_trust_score));
        }
        
        // Channel section
        if let Some(ref channels) = snapshot.channels {
            html.push_str("<h2>Channel Analysis</h2>\n");
            html.push_str(&format!("<p>Recommended Channel: {}</p>\n", channels.recommended_channel));
            
            if !channels.interference_detected.is_empty() {
                html.push_str("<p>Interference Detected:</p>\n<ul>\n");
                for interference in &channels.interference_detected {
                    html.push_str(&format!("<li>Channel {}: {} (severity: {:.2})</li>\n",
                        interference.channel,
                        interference.interference_type,
                        interference.severity
                    ));
                }
                html.push_str("</ul>\n");
            }
        }
        
        // Devices section
        if let Some(ref devices) = snapshot.devices {
            html.push_str("<h2>Device Fingerprinting</h2>\n");
            html.push_str(&format!("<p>Total: {} (Identified: {}, Unidentified: {})</p>\n",
                devices.total_devices,
                devices.identified,
                devices.unidentified
            ));
        }
        
        // HTML footer
        html.push_str("</body>\n</html>");
        
        let mut file = File::create(path)?;
        file.write_all(html.as_bytes())?;
        Ok(())
    }
    
    /// Export to Markdown report
    pub fn export_markdown_report<P: AsRef<Path>>(&self, snapshot: &AnalysisSnapshot, path: P) -> std::io::Result<()> {
        let mut md = String::new();
        
        // Title
        md.push_str("# Zigbee Network Analysis Report\n\n");
        md.push_str(&format!("**Generated:** {:?}\n\n", snapshot.metadata.timestamp));
        md.push_str(&format!("**Duration:** {} seconds\n\n", snapshot.metadata.duration_seconds));
        md.push_str(&format!("**Total Packets:** {}\n\n", snapshot.metadata.total_packets));
        
        // Statistics
        if let Some(ref stats) = snapshot.statistics {
            md.push_str("## Traffic Statistics\n\n");
            md.push_str(&format!("- Total Packets: {}\n", stats.total_packets));
            md.push_str(&format!("- Total Bytes: {}\n", stats.total_bytes));
            md.push_str(&format!("- Average Packet Size: {:.1} bytes\n", stats.avg_packet_size));
            md.push_str(&format!("- Packets per Second: {:.1}\n", stats.packets_per_second));
            md.push_str(&format!("- Unique Devices: {}\n", stats.unique_devices));
            md.push_str("\n");
        }
        
        // Topology
        if let Some(ref topo) = snapshot.topology {
            md.push_str("## Network Topology\n\n");
            md.push_str(&format!("- Devices: {}\n", topo.devices.len()));
            md.push_str(&format!("- Links: {}\n", topo.links.len()));
            md.push_str(&format!("- Networks: {}\n", topo.networks.len()));
            md.push_str("\n");
        }
        
        // Anomalies
        if let Some(ref anomalies) = snapshot.anomalies {
            md.push_str("## Anomalies\n\n");
            md.push_str(&format!("Total: **{}**\n\n", anomalies.total_anomalies));
            md.push_str(&format!("- Critical: {}\n", anomalies.by_severity.critical));
            md.push_str(&format!("- High: {}\n", anomalies.by_severity.high));
            md.push_str(&format!("- Medium: {}\n", anomalies.by_severity.medium));
            md.push_str(&format!("- Low: {}\n", anomalies.by_severity.low));
            md.push_str("\n");
        }
        
        // Security
        if let Some(ref security) = snapshot.security {
            md.push_str("## Security Analysis\n\n");
            md.push_str(&format!("- Total Incidents: {}\n", security.total_incidents));
            md.push_str(&format!("- Encryption Rate: {:.1}%\n", security.avg_encryption_rate * 100.0));
            md.push_str(&format!("- Trust Score: {:.2}/1.0\n", security.avg_trust_score));
            md.push_str("\n");
        }
        
        // Channels
        if let Some(ref channels) = snapshot.channels {
            md.push_str("## Channel Analysis\n\n");
            md.push_str(&format!("**Recommended Channel:** {}\n\n", channels.recommended_channel));
            
            if !channels.interference_detected.is_empty() {
                md.push_str("### Interference Detected\n\n");
                for interference in &channels.interference_detected {
                    md.push_str(&format!("- Channel {}: {} (severity: {:.2})\n",
                        interference.channel,
                        interference.interference_type,
                        interference.severity
                    ));
                }
                md.push_str("\n");
            }
        }
        
        // Devices
        if let Some(ref devices) = snapshot.devices {
            md.push_str("## Device Fingerprinting\n\n");
            md.push_str(&format!("- Total: {}\n", devices.total_devices));
            md.push_str(&format!("- Identified: {}\n", devices.identified));
            md.push_str(&format!("- Unidentified: {}\n", devices.unidentified));
            md.push_str("\n");
        }
        
        let mut file = File::create(path)?;
        file.write_all(md.as_bytes())?;
        Ok(())
    }
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            include_timestamps: true,
            pretty_print_json: true,
            csv_delimiter: ',',
            include_raw_data: false,
        }
    }
}

impl Default for ExportManager {
    fn default() -> Self {
        Self::new()
    }
}