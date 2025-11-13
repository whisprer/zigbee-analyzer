use zigbee_drivers::DriverRegistry;
use zigbee_hal::traits::ZigbeeCapture;
use zigbee_analysis::NetworkTopology;
use std::env;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Zigbee Analyzer - Topology Scanner");
    println!("===================================\n");
    
    let args: Vec<String> = env::args().collect();
    
    // Setup registry
    let mut registry = DriverRegistry::new();
    
    // Add PCAP file if provided
    if args.len() >= 2 {
        let file_path = &args[1];
        registry.add_pcap_file(file_path)?;
    }
    
    // Detect devices
    let devices = registry.detect_devices();
    
    if devices.is_empty() {
        eprintln!("No devices found!");
        eprintln!("Usage: {} [pcap_file]", args[0]);
        return Ok(());
    }
    
    println!("Using: {}\n", devices[0]);
    
    // Create driver
    let mut driver = registry.create_driver(&devices[0])
        .ok_or("Failed to create driver")?;
    
    driver.initialize().await?;
    
    if let Err(e) = driver.set_channel(11).await {
        println!("Could not set channel: {}", e);
    }
    
    // Create topology mapper
    let mut topology = NetworkTopology::new();
    
    println!("Scanning network topology...");
    println!("Capturing packets for 30 seconds (or until PCAP ends)\n");
    
    let start = std::time::Instant::now();
    let mut packet_count = 0;
    
    while start.elapsed() < Duration::from_secs(30) {
        match tokio::time::timeout(Duration::from_secs(1), driver.capture_packet()).await {
            Ok(Ok(packet)) => {
                packet_count += 1;
                
                // Parse and process packet
                if let Ok(parsed) = packet.parse() {
                    topology.process_packet(&parsed, packet.rssi, packet.lqi, packet.channel);
                }
                
                // Progress indicator
                if packet_count % 100 == 0 {
                    print!(".");
                    use std::io::Write;
                    std::io::stdout().flush()?;
                }
            }
            Ok(Err(_)) => {
                // End of file or error
                break;
            }
            Err(_) => {
                // Timeout, continue
            }
        }
    }
    
    println!("\n\nInferring topology structure...");
    topology.infer_topology();
    
    // Print statistics
    let stats = topology.get_statistics();
    println!("\n=== Network Statistics ===");
    println!("Total packets processed: {}", packet_count);
    println!("Total devices discovered: {}", stats.total_devices);
    println!("  Coordinators: {}", stats.coordinators);
    println!("  Routers: {}", stats.routers);
    println!("  End devices: {}", stats.end_devices);
    println!("  Unknown: {}", stats.unknown_devices);
    println!("Total links: {}", stats.total_links);
    println!("Networks (PANs): {}", stats.total_networks);
    println!("Average link quality: {:.1}/255", stats.avg_link_quality);
    
    // Print networks
    println!("\n=== Networks ===");
    for (pan_id, network) in topology.networks() {
        println!("\nPAN ID: 0x{:04x}", pan_id);
        if let Some(coord) = &network.coordinator {
            println!("  Coordinator: {}", coord);
        }
        println!("  Device count: {}", network.device_count);
        if let Some(channel) = network.channel {
            println!("  Channel: {}", channel);
        }
    }
    
    // Print devices
    println!("\n=== Devices ===");
    for device in topology.devices().values() {
        println!("\nDevice: {}", device.mac_addr);
        if let Some(nwk_addr) = device.nwk_addr {
            println!("  Network Address: 0x{:04x}", nwk_addr);
        }
        if let Some(ieee) = device.ieee_addr {
            println!("  IEEE Address: {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                ieee[0], ieee[1], ieee[2], ieee[3], ieee[4], ieee[5], ieee[6], ieee[7]);
        }
        println!("  Type: {:?}", device.device_type);
        if let Some(pan_id) = device.pan_id {
            println!("  PAN ID: 0x{:04x}", pan_id);
        }
        println!("  Packets: {} (TX: {}, RX: {})", 
            device.packet_count, device.total_tx, device.total_rx);
        println!("  Avg RSSI: {:.1} dBm", device.avg_rssi);
        println!("  Avg LQI: {:.1}", device.avg_lqi);
        
        let neighbors = topology.get_neighbors(&device.mac_addr);
        if !neighbors.is_empty() {
            println!("  Neighbors: {}", neighbors.len());
            for neighbor in neighbors.iter().take(5) {
                print!("    - {}", neighbor.mac_addr);
                if let Some(nwk) = neighbor.nwk_addr {
                    print!(" (0x{:04x})", nwk);
                }
                println!();
            }
            if neighbors.len() > 5 {
                println!("    ... and {} more", neighbors.len() - 5);
            }
        }
        
        if !device.profiles.is_empty() {
            print!("  Profiles:");
            for profile in &device.profiles {
                print!(" 0x{:04x}", profile);
            }
            println!();
        }
    }
    
    // Export to DOT format
    let dot_output = topology.export_dot();
    std::fs::write("topology.dot", &dot_output)?;
    println!("\n=== Topology Graph ===");
    println!("Exported to topology.dot");
    println!("Visualize with: dot -Tpng topology.dot -o topology.png");
    
    driver.shutdown().await?;
    
    Ok(())
}