use zigbee_drivers::DriverRegistry;
use zigbee_hal::traits::ZigbeeCapture;
use zigbee_analysis::{DeviceDatabase, DeviceType};
use std::env;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Zigbee Analyzer - Device Fingerprinting");
    println!("========================================\n");
    
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
    
    let mut db = DeviceDatabase::new();
    
    println!("Learning device fingerprints...");
    println!("Analyzing traffic for 30 seconds\n");
    
    let start = std::time::Instant::now();
    let mut packet_count = 0;
    
    while start.elapsed() < Duration::from_secs(30) {
        match tokio::time::timeout(Duration::from_millis(100), driver.capture_packet()).await {
            Ok(Ok(packet)) => {
                packet_count += 1;
                
                if let Ok(parsed) = packet.parse() {
                    db.process_packet(&parsed);
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
    
    println!("\n\n=== Device Fingerprinting Report ===\n");
    
    let stats = db.get_statistics();
    
    println!("Overview:");
    println!("  Total Devices: {}", stats.total_devices);
    println!("  Identified: {}", stats.identified_devices);
    println!("  Unidentified: {}", stats.unidentified_devices);
    println!("  Average Confidence: {:.1}%", stats.avg_confidence * 100.0);
    println!();
    
    println!("Device Types:");
    let mut types: Vec<_> = stats.by_type.iter().collect();
    types.sort_by_key(|(_, count)| std::cmp::Reverse(**count));
    for (dtype, count) in types {
        println!("  {:?}: {}", dtype, count);
    }
    println!();
    
    println!("Manufacturers:");
    let mut mfrs: Vec<_> = stats.by_manufacturer.iter().collect();
    mfrs.sort_by_key(|(_, count)| std::cmp::Reverse(**count));
    for (mfr, count) in mfrs {
        println!("  {}: {}", mfr, count);
    }
    println!();
    
    // Show all identified devices
    println!("=== Identified Devices ===");
    for fingerprint in db.get_all_fingerprints().values() {
        if fingerprint.confidence >= 0.5 {
            println!("\nDevice: {}", fingerprint.mac_addr);
            
            if let Some(ref ieee) = fingerprint.ieee_addr {
                println!("  IEEE: {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                    ieee[0], ieee[1], ieee[2], ieee[3], ieee[4], ieee[5], ieee[6], ieee[7]);
            }
            
            if let Some(nwk) = fingerprint.short_addr {
                println!("  Network Address: 0x{:04x}", nwk);
            }
            
            println!("  Type: {:?}", fingerprint.device_type);
            println!("  Confidence: {:.1}%", fingerprint.confidence * 100.0);
            
            if let Some(ref mfr) = fingerprint.manufacturer {
                println!("  Manufacturer: {}", mfr);
            }
            
            if let Some(ref model) = fingerprint.model {
                println!("  Model: {}", model);
            }
            
            println!("  Power Source: {:?}", fingerprint.power_source);
            
            println!("  Capabilities:");
            if fingerprint.capabilities.can_route {
                println!("    - Can route");
            }
            if fingerprint.capabilities.rx_on_when_idle {
                println!("    - RX always on");
            }
            if fingerprint.capabilities.supports_binding {
                println!("    - Supports binding");
            }
            if fingerprint.capabilities.supports_groups {
                println!("    - Supports groups");
            }
            
            if !fingerprint.protocol.profiles.is_empty() {
                print!("  Profiles:");
                for profile in &fingerprint.protocol.profiles {
                    print!(" 0x{:04x}", profile);
                }
                println!();
            }
            
            if !fingerprint.protocol.server_clusters.is_empty() {
                print!("  Server Clusters:");
                for cluster in fingerprint.protocol.server_clusters.iter().take(10) {
                    print!(" 0x{:04x}", cluster);
                }
                if fingerprint.protocol.server_clusters.len() > 10 {
                    print!(" ... +{}", fingerprint.protocol.server_clusters.len() - 10);
                }
                println!();
            }
            
            if !fingerprint.protocol.client_clusters.is_empty() {
                print!("  Client Clusters:");
                for cluster in fingerprint.protocol.client_clusters.iter().take(10) {
                    print!(" 0x{:04x}", cluster);
                }
                if fingerprint.protocol.client_clusters.len() > 10 {
                    print!(" ... +{}", fingerprint.protocol.client_clusters.len() - 10);
                }
                println!();
            }
            
            println!("  Behavior:");
            println!("    Avg packet size: {:.1} bytes", fingerprint.behavior.avg_packet_size);
            println!("    Communication peers: {}", fingerprint.behavior.typical_peers.len());
            println!("    Encryption rate: {:.1}%", fingerprint.protocol.encryption_rate * 100.0);
            
            println!("  Statistics:");
            println!("    Packets: {}", fingerprint.packet_count);
        }
    }
    
    // Show unidentified devices
    let unidentified = db.get_unidentified_devices();
    if !unidentified.is_empty() {
        println!("\n=== Unidentified Devices ===");
        for fingerprint in unidentified {
            println!("\nDevice: {}", fingerprint.mac_addr);
            
            if let Some(ref ieee) = fingerprint.ieee_addr {
                println!("  IEEE: {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                    ieee[0], ieee[1], ieee[2], ieee[3], ieee[4], ieee[5], ieee[6], ieee[7]);
            }
            
            println!("  Type: {:?} (confidence: {:.1}%)", fingerprint.device_type, fingerprint.confidence * 100.0);
            
            if let Some(ref mfr) = fingerprint.manufacturer {
                println!("  Manufacturer: {}", mfr);
            }
            
            println!("  Observed characteristics:");
            if !fingerprint.protocol.profiles.is_empty() {
                print!("    Profiles:");
                for profile in &fingerprint.protocol.profiles {
                    print!(" 0x{:04x}", profile);
                }
                println!();
            }
            
            if !fingerprint.protocol.server_clusters.is_empty() {
                print!("    Clusters:");
                for cluster in fingerprint.protocol.server_clusters.iter().take(5) {
                    print!(" 0x{:04x}", cluster);
                }
                println!();
            }
            
            println!("    Packets: {}", fingerprint.packet_count);
        }
    }
    
    // Export to JSON
    if args.len() >= 3 && args[2] == "--export" {
        let json = db.export_json()?;
        std::fs::write("fingerprints.json", json)?;
        println!("\n✓ Fingerprints exported to fingerprints.json");
    }
    
    driver.shutdown().await?;
    
    Ok(())
}