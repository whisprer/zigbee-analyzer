use zigbee_drivers::DriverRegistry;
use zigbee_hal::traits::ZigbeeCapture;
use zigbee_analysis::{SecurityAnalyzer, IncidentSeverity};
use std::env;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Zigbee Analyzer - Security Audit");
    println!("=================================\n");
    
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
    
    let mut analyzer = SecurityAnalyzer::new();
    
    println!("Performing security audit...");
    println!("Analyzing network traffic for security issues\n");
    
    let start = std::time::Instant::now();
    let mut packet_count = 0;
    
    while start.elapsed() < Duration::from_secs(30) {
        match tokio::time::timeout(Duration::from_millis(100), driver.capture_packet()).await {
            Ok(Ok(packet)) => {
                packet_count += 1;
                
                if let Ok(parsed) = packet.parse() {
                    analyzer.process_packet(&parsed);
                }
                
                if packet_count % 100 == 0 {
                    print!(".");
                    use std::io::Write;
                    std::io::stdout().flush()?;
                }
            }
            Ok(Err(_)) => break,
            Err(_) => {}
        }
    }
    
    println!("\n\n=== Security Audit Report ===\n");
    
    let stats = analyzer.get_statistics();
    
    println!("Overview:");
    println!("  Packets Analyzed: {}", packet_count);
    println!("  Networks Found: {}", stats.networks_analyzed);
    println!("  Devices Found: {}", stats.devices_analyzed);
    println!("  Secured Devices: {}", stats.secured_devices);
    println!("  Unsecured Devices: {}", stats.unsecured_devices);
    println!("  Average Encryption Rate: {:.1}%", stats.avg_encryption_rate * 100.0);
    println!("  Average Trust Score: {:.1}/1.0", stats.avg_trust_score);
    println!();
    
    println!("Security Incidents:");
    println!("  Total: {}", stats.total_incidents);
    println!("  Critical: {}", stats.critical_incidents);
    println!("  High: {}", stats.high_incidents);
    println!("  Medium: {}", stats.medium_incidents);
    println!("  Low: {}", stats.low_incidents);
    println!("  Info: {}", stats.info_incidents);
    println!();
    
    println!("Join Activity:");
    println!("  Total Attempts: {}", stats.join_attempts);
    println!("  Successful: {}", stats.successful_joins);
    println!("  Failed: {}", stats.failed_joins);
    println!();
    
    println!("Key Events: {}", stats.key_events);
    println!();
    
    // Show critical and high severity incidents
    let critical_high = analyzer.get_incidents_by_severity(IncidentSeverity::High);
    if !critical_high.is_empty() {
        println!("=== Critical & High Severity Incidents ===");
        for incident in critical_high.iter().take(10) {
            println!("\n[{:?}] {:?}", incident.severity, incident.incident_type);
            println!("  {}", incident.description);
            if let Some(device) = &incident.affected_device {
                println!("  Device: {}", device);
            }
            if !incident.evidence.is_empty() {
                println!("  Evidence:");
                for evidence in &incident.evidence {
                    println!("    - {}", evidence);
                }
            }
            if !incident.mitigations.is_empty() {
                println!("  Recommended Actions:");
                for mitigation in &incident.mitigations {
                    println!("    - {}", mitigation);
                }
            }
        }
        println!();
    }
    
    // Network assessments
    println!("=== Network Security Assessments ===");
    for (pan_id, _) in analyzer.networks.iter() {
        if let Some(assessment) = analyzer.get_network_assessment(*pan_id) {
            println!("\nPAN ID: 0x{:04x}", assessment.pan_id);
            println!("  Security Score: {:.1}/100", assessment.overall_score);
            println!("  Security Level: {:?}", assessment.security_level);
            println!("  Encryption Rate: {:.1}%", assessment.encryption_rate * 100.0);
            
            if !assessment.issues.is_empty() {
                println!("  Issues:");
                for issue in &assessment.issues {
                    println!("    ⚠ {}", issue);
                }
            }
            
            if !assessment.recommendations.is_empty() {
                println!("  Recommendations:");
                for rec in &assessment.recommendations {
                    println!("    ➜ {}", rec);
                }
            }
        }
    }
    
    // Device assessments (top 10 by trust score)
    println!("\n=== Device Security Assessments ===");
    let mut devices: Vec<_> = analyzer.devices.values().collect();
    devices.sort_by(|a, b| b.trust_score.partial_cmp(&a.trust_score).unwrap());
    
    for device in devices.iter().take(10) {
        if let Some(assessment) = analyzer.get_device_assessment(&device.mac_addr) {
            println!("\nDevice: {}", assessment.mac_addr);
            println!("  Trust Score: {:.2}/1.0", assessment.trust_score);
            println!("  Encryption: {}", if assessment.encryption_enabled { "Enabled" } else { "Disabled" });
            println!("  Security Level: {:?}", assessment.security_level);
            
            if !assessment.issues.is_empty() {
                println!("  Issues:");
                for issue in &assessment.issues {
                    println!("    ⚠ {}", issue);
                }
            }
            
            if !assessment.recommendations.is_empty() {
                println!("  Recommendations:");
                for rec in &assessment.recommendations {
                    println!("    ➜ {}", rec);
                }
            }
        }
    }
    
    // Key events
    if stats.key_events > 0 {
        println!("\n=== Key Events ===");
        for event in analyzer.get_key_events().iter().take(10) {
            println!("{:?}: {} - {}", event.event_type, event.device, event.description);
        }
    }
    
    driver.shutdown().await?;
    
    Ok(())
}