use zigbee_drivers::DriverRegistry;
use zigbee_hal::traits::ZigbeeCapture;
use zigbee_analysis::{AnomalyDetector, Severity, DetectorConfig};
use std::env;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Zigbee Analyzer - Anomaly Detection");
    println!("====================================\n");
    
    let args: Vec<String> = env::args().collect();
    
    let mut registry = DriverRegistry::new();
    
    if args.len() >= 2 {
        registry.add_pcap_file(&args[1])?;
    }
    
    let devices = registry.detect_devices();
    
    if devices.is_empty() {
        eprintln!("No devices found!");
        eprintln!("Usage: {} [pcap_file]", args[0]);
        return Ok(());
    }
    
    println!("Using: {}\n", devices[0]);
    
    let mut driver = registry.create_driver(&devices[0])
        .ok_or("Failed to create driver")?;
    
    driver.initialize().await?;
    let _ = driver.set_channel(11).await;
    
    // Create detector with custom config
    let config = DetectorConfig {
        traffic_spike_threshold: 2.5,
        behavior_change_threshold: 0.7,
        min_baseline_samples: 50,
        anomaly_retention_hours: 24,
        enable_traffic_detection: true,
        enable_security_detection: true,
        enable_behavior_detection: true,
        enable_protocol_detection: true,
    };
    
    let mut detector = AnomalyDetector::with_config(config);
    
    println!("Monitoring network for anomalies...");
    println!("Detection enabled:");
    println!("  ✓ Traffic anomalies");
    println!("  ✓ Security threats");
    println!("  ✓ Behavioral changes");
    println!("  ✓ Protocol violations");
    println!();
    
    let start = std::time::Instant::now();
    let mut packet_count = 0;
    let mut last_report = std::time::Instant::now();
    
    while start.elapsed() < Duration::from_secs(60) {
        match tokio::time::timeout(Duration::from_millis(100), driver.capture_packet()).await {
            Ok(Ok(packet)) => {
                packet_count += 1;
                
                if let Ok(parsed) = packet.parse() {
                    detector.process_packet(&parsed, packet.rssi, packet.channel);
                }
                
                // Print alerts in real-time for high/critical severity
                let recent = detector.get_recent_anomalies(1);
                if let Some(anomaly) = recent.first() {
                    if anomaly.severity >= Severity::High {
                        print_alert(anomaly);
                    }
                }
            }
            Ok(Err(_)) => break,
            Err(_) => {}
        }
        
        // Print summary every 10 seconds
        if last_report.elapsed() >= Duration::from_secs(10) {
            print_summary(&detector, packet_count);
            last_report = std::time::Instant::now();
        }
    }
    
    println!("\n\n=== Final Anomaly Report ===\n");
    print_detailed_report(&detector, packet_count);
    
    driver.shutdown().await?;
    
    Ok(())
}

fn print_alert(anomaly: &zigbee_analysis::Anomaly) {
    let severity_str = match anomaly.severity {
        Severity::Critical => "🔴 CRITICAL",
        Severity::High => "🟠 HIGH",
        Severity::Medium => "🟡 MEDIUM",
        Severity::Low => "🟢 LOW",
    };
    
    println!("\n{} - {:?}", severity_str, anomaly.anomaly_type);
    println!("  {}", anomaly.description);
    if let Some(device) = &anomaly.affected_device {
        println!("  Device: {}", device);
    }
    println!("  Confidence: {:.0}%", anomaly.confidence * 100.0);
}

fn print_summary(detector: &AnomalyDetector, packets: u64) {
    let stats = detector.get_statistics();
    
    println!("\r[Status] Packets: {} | Anomalies: {} (C:{} H:{} M:{} L:{})",
        packets,
        stats.total_anomalies,
        stats.critical,
        stats.high,
        stats.medium,
        stats.low
    );
}

fn print_detailed_report(detector: &AnomalyDetector, packets: u64) {
    let stats = detector.get_statistics();
    
    println!("Packets Processed: {}", packets);
    println!("Total Anomalies: {}", stats.total_anomalies);
    println!("Anomaly Rate: {:.4}%", stats.anomaly_rate * 100.0);
    println!();
    
    println!("Severity Breakdown:");
    println!("  Critical: {} ({:.1}%)", stats.critical, 
        stats.critical as f32 / stats.total_anomalies.max(1) as f32 * 100.0);
    println!("  High:     {} ({:.1}%)", stats.high,
        stats.high as f32 / stats.total_anomalies.max(1) as f32 * 100.0);
    println!("  Medium:   {} ({:.1}%)", stats.medium,
        stats.medium as f32 / stats.total_anomalies.max(1) as f32 * 100.0);
    println!("  Low:      {} ({:.1}%)", stats.low,
        stats.low as f32 / stats.total_anomalies.max(1) as f32 * 100.0);
    println!();
    
    println!("Anomaly Types:");
    let mut types: Vec<_> = stats.by_type.iter().collect();
    types.sort_by(|a, b| b.1.cmp(a.1));
    for (atype, count) in types {
        println!("  {:?}: {}", atype, count);
    }
    println!();
    
    // Show all critical and high severity anomalies
    let critical_and_high = detector.get_anomalies_by_severity(Severity::High);
    if !critical_and_high.is_empty() {
        println!("=== Critical & High Severity Anomalies ===");
        for anomaly in critical_and_high.iter().take(20) {
            println!("\n[{}] {:?}", 
                match anomaly.severity {
                    Severity::Critical => "CRITICAL",
                    Severity::High => "HIGH",
                    _ => "OTHER",
                },
                anomaly.anomaly_type
            );
            println!("  {}", anomaly.description);
            if let Some(device) = &anomaly.affected_device {
                println!("  Device: {}", device);
            }
            println!("  Confidence: {:.0}%", anomaly.confidence * 100.0);
            if !anomaly.evidence.is_empty() {
                println!("  Evidence:");
                for evidence in &anomaly.evidence {
                    println!("    - {}", evidence);
                }
            }
        }
    }
    
    // Show most recent anomalies
    println!("\n=== Recent Anomalies (Last 10) ===");
    for (i, anomaly) in detector.get_recent_anomalies(10).iter().enumerate() {
        println!("{}. [{:?}] {:?} - {}",
            i + 1,
            anomaly.severity,
            anomaly.anomaly_type,
            anomaly.description
        );
    }
}