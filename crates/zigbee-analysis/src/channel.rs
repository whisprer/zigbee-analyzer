use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, Duration};

/// Channel analyzer for spectrum monitoring
pub struct ChannelAnalyzer {
    channels: HashMap<u8, ChannelMetrics>,
    interference_detectors: HashMap<u8, InterferenceDetector>,
    scan_history: Vec<ChannelScan>,
    start_time: SystemTime,
}

/// Metrics for a single channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelMetrics {
    pub channel: u8,
    pub frequency_mhz: u16,
    
    // Signal measurements
    pub avg_rssi: f32,
    pub min_rssi: i8,
    pub max_rssi: i8,
    pub rssi_stddev: f32,
    
    pub avg_lqi: f32,
    pub min_lqi: u8,
    pub max_lqi: u8,
    
    // Traffic measurements
    pub packet_count: u64,
    pub byte_count: u64,
    pub packets_per_second: f32,
    pub utilization: f32, // 0.0 - 1.0
    
    // Quality metrics
    pub noise_floor: i8,
    pub snr: f32, // Signal-to-Noise Ratio
    pub error_rate: f32,
    pub retry_rate: f32,
    
    // Interference detection
    pub interference_score: f32, // 0.0 (none) - 1.0 (severe)
    pub interference_type: InterferenceType,
    
    // Timing
    pub first_seen: SystemTime,
    pub last_seen: SystemTime,
    pub active_time: Duration,
    
    // Quality score (0-100)
    pub quality_score: f32,
}

/// Interference detector for a channel
struct InterferenceDetector {
    rssi_samples: Vec<(SystemTime, i8)>,
    packet_intervals: Vec<Duration>,
    periodic_patterns: Vec<PeriodicPattern>,
    last_update: SystemTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InterferenceType {
    None,
    WiFi,           // WiFi co-channel interference
    Microwave,      // Microwave oven interference
    Bluetooth,      // Bluetooth interference
    Cordless,       // Cordless phone
    Generic,        // Unknown periodic interference
    Narrowband,     // Narrowband interference
    Wideband,       // Wideband noise
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PeriodicPattern {
    period_ms: f32,
    confidence: f32,
    last_occurrence: SystemTime,
}

/// A channel scan result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelScan {
    pub timestamp: SystemTime,
    pub channels: Vec<ChannelMetrics>,
}

/// Channel recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelRecommendation {
    pub recommended_channel: u8,
    pub quality_score: f32,
    pub reason: String,
    pub alternatives: Vec<(u8, f32)>, // (channel, score) pairs
}

impl ChannelAnalyzer {
    pub fn new() -> Self {
        Self {
            channels: HashMap::new(),
            interference_detectors: HashMap::new(),
            scan_history: Vec::new(),
            start_time: SystemTime::now(),
        }
    }
    
    /// Process a packet on a specific channel
    pub fn process_packet(&mut self, channel: u8, rssi: i8, lqi: u8, packet_len: usize, had_errors: bool) {
        let now = SystemTime::now();
        
        // Get or create channel metrics
        let metrics = self.channels.entry(channel).or_insert_with(|| {
            ChannelMetrics::new(channel)
        });
        
        // Update basic metrics
        metrics.update(rssi, lqi, packet_len, had_errors, now);
        
        // Get or create interference detector
        let detector = self.interference_detectors.entry(channel)
            .or_insert_with(|| InterferenceDetector::new());
        
        detector.process_sample(rssi, now);
        
        // Analyze interference
        let (interference_score, interference_type) = detector.analyze_interference();
        metrics.interference_score = interference_score;
        metrics.interference_type = interference_type;
        
        // Calculate overall quality score
        metrics.calculate_quality_score();
    }
    
    /// Get metrics for a specific channel
    pub fn get_channel_metrics(&self, channel: u8) -> Option<&ChannelMetrics> {
        self.channels.get(&channel)
    }
    
    /// Get all channel metrics
    pub fn get_all_channels(&self) -> &HashMap<u8, ChannelMetrics> {
        &self.channels
    }
    
    /// Get best channel based on quality scores
    pub fn get_best_channel(&self) -> Option<u8> {
        self.channels.iter()
            .max_by(|a, b| a.1.quality_score.partial_cmp(&b.1.quality_score).unwrap())
            .map(|(ch, _)| *ch)
    }
    
    /// Get worst channel
    pub fn get_worst_channel(&self) -> Option<u8> {
        self.channels.iter()
            .min_by(|a, b| a.1.quality_score.partial_cmp(&b.1.quality_score).unwrap())
            .map(|(ch, _)| *ch)
    }
    
    /// Get channels sorted by quality
    pub fn get_channels_by_quality(&self) -> Vec<(u8, f32)> {
        let mut channels: Vec<_> = self.channels.iter()
            .map(|(ch, metrics)| (*ch, metrics.quality_score))
            .collect();
        channels.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        channels
    }
    
    /// Get channels with interference
    pub fn get_channels_with_interference(&self, threshold: f32) -> Vec<(u8, InterferenceType, f32)> {
        self.channels.iter()
            .filter(|(_, m)| m.interference_score > threshold)
            .map(|(ch, m)| (*ch, m.interference_type, m.interference_score))
            .collect()
    }
    
    /// Recommend best channel based on analysis
    pub fn recommend_channel(&self) -> ChannelRecommendation {
        let sorted = self.get_channels_by_quality();
        
        if sorted.is_empty() {
            return ChannelRecommendation {
                recommended_channel: 11,
                quality_score: 0.0,
                reason: "No channel data available. Defaulting to channel 11.".to_string(),
                alternatives: Vec::new(),
            };
        }
        
        let (best_channel, best_score) = sorted[0];
        let alternatives: Vec<_> = sorted.iter().skip(1).take(3).copied().collect();
        
        // Generate reason
        let reason = if let Some(metrics) = self.channels.get(&best_channel) {
            let mut reasons = Vec::new();
            
            if metrics.interference_score < 0.2 {
                reasons.push("low interference");
            }
            if metrics.utilization < 0.3 {
                reasons.push("low utilization");
            }
            if metrics.avg_rssi > -60.0 {
                reasons.push("strong signal");
            }
            if metrics.avg_lqi > 200.0 {
                reasons.push("high link quality");
            }
            
            if reasons.is_empty() {
                format!("Channel {} selected as best available option", best_channel)
            } else {
                format!("Channel {} recommended: {}", best_channel, reasons.join(", "))
            }
        } else {
            format!("Channel {} has highest quality score", best_channel)
        };
        
        ChannelRecommendation {
            recommended_channel: best_channel,
            quality_score: best_score,
            reason,
            alternatives,
        }
    }
    
    /// Perform a channel scan snapshot
    pub fn snapshot(&mut self) -> ChannelScan {
        let scan = ChannelScan {
            timestamp: SystemTime::now(),
            channels: self.channels.values().cloned().collect(),
        };
        
        self.scan_history.push(scan.clone());
        
        // Keep only last 100 scans
        if self.scan_history.len() > 100 {
            self.scan_history.remove(0);
        }
        
        scan
    }
    
    /// Get scan history
    pub fn get_scan_history(&self) -> &[ChannelScan] {
        &self.scan_history
    }
    
    /// Check for WiFi interference on specific channels
    pub fn detect_wifi_interference(&self) -> Vec<u8> {
        // WiFi channels overlap with Zigbee
        // WiFi ch 1 (2412 MHz) overlaps with Zigbee 11-14
        // WiFi ch 6 (2437 MHz) overlaps with Zigbee 16-19
        // WiFi ch 11 (2462 MHz) overlaps with Zigbee 21-24
        
        self.channels.iter()
            .filter(|(_, m)| m.interference_type == InterferenceType::WiFi)
            .map(|(ch, _)| *ch)
            .collect()
    }
    
    /// Get channel overlap analysis
    pub fn analyze_channel_overlap(&self, channel: u8) -> ChannelOverlapAnalysis {
        let frequency = channel_to_frequency(channel);
        
        // Zigbee channels are 5 MHz apart, but have ~2 MHz bandwidth
        // So adjacent channels can interfere
        let adjacent_channels: Vec<u8> = (11..=26)
            .filter(|ch| {
                let ch_freq = channel_to_frequency(*ch);
                let diff = (ch_freq as i32 - frequency as i32).abs();
                diff <= 5 && *ch != channel
            })
            .collect();
        
        let mut overlapping_traffic = 0u64;
        for adj_ch in &adjacent_channels {
            if let Some(metrics) = self.channels.get(adj_ch) {
                overlapping_traffic += metrics.packet_count;
            }
        }
        
        ChannelOverlapAnalysis {
            channel,
            frequency,
            adjacent_channels,
            overlapping_traffic,
        }
    }
    
    /// Get spectrum visualization data
    pub fn get_spectrum_data(&self) -> SpectrumData {
        let mut channels_data = Vec::new();
        
        for channel in 11..=26 {
            let metrics = self.channels.get(&channel);
            
            channels_data.push(SpectrumChannelData {
                channel,
                frequency_mhz: channel_to_frequency(channel),
                rssi: metrics.map(|m| m.avg_rssi).unwrap_or(-100.0),
                lqi: metrics.map(|m| m.avg_lqi).unwrap_or(0.0),
                utilization: metrics.map(|m| m.utilization).unwrap_or(0.0),
                packet_count: metrics.map(|m| m.packet_count).unwrap_or(0),
                quality_score: metrics.map(|m| m.quality_score).unwrap_or(0.0),
                has_interference: metrics.map(|m| m.interference_score > 0.3).unwrap_or(false),
            });
        }
        
        SpectrumData {
            channels: channels_data,
            timestamp: SystemTime::now(),
        }
    }
}

impl ChannelMetrics {
    fn new(channel: u8) -> Self {
        let now = SystemTime::now();
        
        Self {
            channel,
            frequency_mhz: channel_to_frequency(channel),
            avg_rssi: -100.0,
            min_rssi: 0,
            max_rssi: -128,
            rssi_stddev: 0.0,
            avg_lqi: 0.0,
            min_lqi: 255,
            max_lqi: 0,
            packet_count: 0,
            byte_count: 0,
            packets_per_second: 0.0,
            utilization: 0.0,
            noise_floor: -100,
            snr: 0.0,
            error_rate: 0.0,
            retry_rate: 0.0,
            interference_score: 0.0,
            interference_type: InterferenceType::None,
            first_seen: now,
            last_seen: now,
            active_time: Duration::from_secs(0),
            quality_score: 0.0,
        }
    }
    
    fn update(&mut self, rssi: i8, lqi: u8, packet_len: usize, had_errors: bool, now: SystemTime) {
        self.packet_count += 1;
        self.byte_count += packet_len as u64;
        self.last_seen = now;
        
        // Update active time
        if let Ok(duration) = now.duration_since(self.first_seen) {
            self.active_time = duration;
            
            // Update packets per second
            let seconds = duration.as_secs_f32();
            if seconds > 0.0 {
                self.packets_per_second = self.packet_count as f32 / seconds;
                
                // Estimate utilization (Zigbee is ~250 kbps)
                let bits_sent = self.byte_count * 8;
                let max_bits = (250_000.0 * seconds) as u64;
                self.utilization = (bits_sent as f32 / max_bits as f32).min(1.0);
            }
        }
        
        // Update RSSI statistics
        let alpha = 0.1; // Smoothing factor
        if self.packet_count == 1 {
            self.avg_rssi = rssi as f32;
        } else {
            self.avg_rssi = self.avg_rssi * (1.0 - alpha) + (rssi as f32) * alpha;
        }
        self.min_rssi = self.min_rssi.max(rssi);
        self.max_rssi = self.max_rssi.max(rssi);
        
        // Update LQI statistics
        if self.packet_count == 1 {
            self.avg_lqi = lqi as f32;
        } else {
            self.avg_lqi = self.avg_lqi * (1.0 - alpha) + (lqi as f32) * alpha;
        }
        self.min_lqi = self.min_lqi.min(lqi);
        self.max_lqi = self.max_lqi.max(lqi);
        
        // Update error rate
        if had_errors {
            self.error_rate = self.error_rate * 0.95 + 0.05;
        } else {
            self.error_rate = self.error_rate * 0.95;
        }
    }
    
    fn calculate_quality_score(&mut self) {
        // Calculate quality score (0-100) based on multiple factors
        
        // RSSI score (0-100): -90 dBm = 0, -30 dBm = 100
        let rssi_score = ((self.avg_rssi + 90.0) / 60.0 * 100.0).clamp(0.0, 100.0);
        
        // LQI score (0-100): 0 = 0, 255 = 100
        let lqi_score = (self.avg_lqi / 255.0 * 100.0).clamp(0.0, 100.0);
        
        // Utilization score (0-100): lower is better
        let util_score = (1.0 - self.utilization) * 100.0;
        
        // Interference score (0-100): lower is better
        let interference_score = (1.0 - self.interference_score) * 100.0;
        
        // Error rate score (0-100): lower is better
        let error_score = (1.0 - self.error_rate) * 100.0;
        
        // Weighted average
        self.quality_score = (
            rssi_score * 0.25 +
            lqi_score * 0.25 +
            util_score * 0.20 +
            interference_score * 0.20 +
            error_score * 0.10
        ).clamp(0.0, 100.0);
    }
}

impl InterferenceDetector {
    fn new() -> Self {
        Self {
            rssi_samples: Vec::new(),
            packet_intervals: Vec::new(),
            periodic_patterns: Vec::new(),
            last_update: SystemTime::now(),
        }
    }
    
    fn process_sample(&mut self, rssi: i8, timestamp: SystemTime) {
        // Keep samples for last 60 seconds
        let cutoff = timestamp - Duration::from_secs(60);
        self.rssi_samples.retain(|(time, _)| *time > cutoff);
        
        self.rssi_samples.push((timestamp, rssi));
        
        // Calculate interval if we have a previous sample
        if self.rssi_samples.len() > 1 {
            let prev_time = self.rssi_samples[self.rssi_samples.len() - 2].0;
            if let Ok(interval) = timestamp.duration_since(prev_time) {
                self.packet_intervals.push(interval);
                
                // Keep last 100 intervals
                if self.packet_intervals.len() > 100 {
                    self.packet_intervals.remove(0);
                }
            }
        }
        
        self.last_update = timestamp;
    }
    
    fn analyze_interference(&mut self) -> (f32, InterferenceType) {
        if self.rssi_samples.len() < 10 {
            return (0.0, InterferenceType::None);
        }
        
        // Calculate RSSI variance
        let rssi_values: Vec<f32> = self.rssi_samples.iter().map(|(_, r)| *r as f32).collect();
        let mean = rssi_values.iter().sum::<f32>() / rssi_values.len() as f32;
        let variance = rssi_values.iter()
            .map(|r| (r - mean).powi(2))
            .sum::<f32>() / rssi_values.len() as f32;
        let stddev = variance.sqrt();
        
        // High variance suggests interference
        let variance_score = (stddev / 20.0).min(1.0);
        
        // Check for periodic patterns in intervals
        let periodic_score = self.detect_periodic_patterns();
        
        // Determine interference type based on patterns
        let interference_type = if periodic_score > 0.7 {
            // Check period length to classify
            if let Some(pattern) = self.periodic_patterns.first() {
                if pattern.period_ms > 100.0 && pattern.period_ms < 200.0 {
                    InterferenceType::Microwave // ~5-10 Hz pattern
                } else if pattern.period_ms > 10.0 && pattern.period_ms < 50.0 {
                    InterferenceType::Bluetooth // ~20-100 Hz hopping
                } else {
                    InterferenceType::Generic
                }
            } else {
                InterferenceType::Generic
            }
        } else if variance_score > 0.6 {
            if stddev > 15.0 {
                InterferenceType::Wideband
            } else {
                InterferenceType::WiFi // WiFi often shows as moderate variance
            }
        } else {
            InterferenceType::None
        };
        
        // Overall interference score
        let interference_score = (variance_score * 0.6 + periodic_score * 0.4).clamp(0.0, 1.0);
        
        (interference_score, interference_type)
    }
    
    fn detect_periodic_patterns(&mut self) -> f32 {
        if self.packet_intervals.len() < 20 {
            return 0.0;
        }
        
        // Simple autocorrelation to detect periodicity
        // This is a simplified version - a real implementation would use FFT
        
        let intervals_ms: Vec<f32> = self.packet_intervals.iter()
            .map(|d| d.as_secs_f32() * 1000.0)
            .collect();
        
        let mean = intervals_ms.iter().sum::<f32>() / intervals_ms.len() as f32;
        let variance = intervals_ms.iter()
            .map(|i| (i - mean).powi(2))
            .sum::<f32>() / intervals_ms.len() as f32;
        
        if variance < 1.0 {
            // Very regular intervals suggest strong periodicity
            return 0.8;
        }
        
        // Calculate coefficient of variation
        let cv = variance.sqrt() / mean;
        
        // Lower CV means more periodic
        let periodicity_score = (1.0 - cv.min(1.0)).max(0.0);
        
        periodicity_score
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelOverlapAnalysis {
    pub channel: u8,
    pub frequency: u16,
    pub adjacent_channels: Vec<u8>,
    pub overlapping_traffic: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpectrumData {
    pub channels: Vec<SpectrumChannelData>,
    pub timestamp: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpectrumChannelData {
    pub channel: u8,
    pub frequency_mhz: u16,
    pub rssi: f32,
    pub lqi: f32,
    pub utilization: f32,
    pub packet_count: u64,
    pub quality_score: f32,
    pub has_interference: bool,
}

/// Convert Zigbee channel to frequency in MHz
fn channel_to_frequency(channel: u8) -> u16 {
    2405 + ((channel - 11) as u16) * 5
}

impl Default for ChannelAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}