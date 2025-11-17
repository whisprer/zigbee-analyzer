pub mod topology;
pub mod statistics;
pub mod device_db;
pub mod security;
pub mod channel;
pub mod anomaly;
pub mod export;

// Export from topology
pub use topology::{TopologyMap, Device, DeviceType as TopologyDeviceType, Link, Network, TopologyStatistics};

// Export from statistics
pub use statistics::{TrafficStatistics, StatisticsSummary, ChannelStats, DeviceStats, ProtocolStats};

// Export from channel
pub use channel::{ChannelAnalyzer, ChannelMetrics, InterferenceType, ChannelRecommendation, SpectrumData};

// Export from anomaly
pub use anomaly::{AnomalyDetector, Anomaly, AnomalyType, Severity as AnomalySeverity, DetectorConfig, AnomalyStatistics};

// Export from security
pub use security::{SecurityAnalyzer, SecurityThreat, ThreatType, ThreatSeverity, SecurityIncident, SecurityStatistics, SecurityConfig, DeviceSecurity};

// Export from device_db
pub use device_db::{DeviceDatabase, DeviceFingerprint, DeviceType, DeviceStatistics};

// Export from export
pub use export::{ExportManager, ExportConfig, ExportFormat, AnalysisSnapshot};

// Re-export topology::DeviceType for UI compatibility
pub mod topology_types {
    pub use crate::topology::DeviceType;
}
