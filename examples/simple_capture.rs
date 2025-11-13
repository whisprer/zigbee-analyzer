use zigbee_drivers::DriverRegistry;
use zigbee_hal::traits::ZigbeeCapture;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    println!("Zigbee Analyzer - Simple Capture Example");
    println!("==========================================\n");
    
    let args: Vec<String> = env::args().collect();
    
    // Create registry
    let mut registry = DriverRegistry::new();
    
    // Check if user specified a PCAP file
    if args.len() >= 2 {
        let file_path = &args[1];
        println!("Loading PCAP file: {}\n", file_path);
        
        match registry.add_pcap_file(file_path) {
            Ok(_) => println!("PCAP file added successfully"),
            Err(e) => {
                eprintln!("Error adding PCAP file: {}", e);
                return Ok(());
            }
        }
    }
    
    // Detect available devices (hardware + PCAP files)
    let devices = registry.detect_devices();
    
    if devices.is_empty() {
        println!("No Zigbee devices or PCAP files found!");
        println!("\nSupported hardware devices:");
        for driver in registry.list_drivers() {
            println!("  - {} ({:04x}:{:04x})", driver.description, driver.vid, driver.pid);
        }
        println!("\nUsage: {} [pcap_file]", args.get(0).unwrap_or(&"program".to_string()));
        return Ok(());
    }
    
    println!("Found {} device(s):", devices.len());
    for (i, device) in devices.iter().enumerate() {
        println!("  {}. {}", i + 1, device);
    }
    
    // Use the first device
    let device = &devices[0];
    println!("\nUsing: {}", device);
    
    let mut driver = registry.create_driver(device)
        .ok_or("Failed to create driver")?;
    
    println!("Initializing...");
    driver.initialize().await?;
    println!("Device initialized successfully!");
    
    // Print capabilities
    let caps = driver.capabilities();
    println!("\nDevice Capabilities:");
    println!("  Packet injection: {}", caps.packet_injection);
    println!("  Promiscuous mode: {}", caps.promiscuous_mode);
    println!("  Hardware timestamps: {}", caps.hardware_timestamps);
    println!("  Supported channels: {:?}", caps.supported_channels);
    println!("  Max sample rate: {} packets/sec", caps.max_sample_rate);
    
    // Set channel (if hardware)
    if device.device_type == zigbee_drivers::registry::DeviceType::Hardware {
        let channel = 11;
        println!("\nSetting channel to {}...", channel);
        driver.set_channel(channel).await?;
    }
    
    // Capture packets
    println!("\nCapturing packets (press Ctrl+C to stop)...\n");
    
    let mut packet_count = 0;
    let max_packets = if device.device_type == zigbee_drivers::registry::DeviceType::PcapFile {
        100 // Limit for PCAP playback
    } else {
        usize::MAX
    };
    
    loop {
        if packet_count >= max_packets {
            break;
        }
        
        match driver.capture_packet().await {
            Ok(packet) => {
                packet_count += 1;
                
                println!("Packet #{}", packet_count);
                println!("  Channel: {}", packet.channel);
                println!("  RSSI: {} dBm", packet.rssi);
                println!("  LQI: {}", packet.lqi);
                println!("  Length: {} bytes", packet.data.len());
                
                // Try to parse the packet
                match packet.parse() {
                    Ok(parsed) => {
                        println!("  Frame Type: {:?}", parsed.mac.frame_control.frame_type);
                        println!("  Src: {}", parsed.mac.src_addr);
                        println!("  Dst: {}", parsed.mac.dest_addr);
                        
                        if let Some(nwk) = parsed.network {
                            println!("  NWK Src: 0x{:04x}", nwk.src_addr);
                            println!("  NWK Dst: 0x{:04x}", nwk.dest_addr);
                        }
                        
                        if let Some(aps) = parsed.aps {
                            if let Some(profile) = aps.profile_id {
                                println!("  Profile: 0x{:04x}", profile);
                            }
                            if let Some(cluster) = aps.cluster_id {
                                println!("  Cluster: 0x{:04x}", cluster);
                            }
                        }
                    }
                    Err(e) => {
                        println!("  Parse error: {}", e);
                    }
                }
                
                // Print raw bytes (first 16 bytes)
                print!("  Raw: ");
                for (i, byte) in packet.data.iter().take(16).enumerate() {
                    print!("{:02x} ", byte);
                    if i == 7 {
                        print!(" ");
                    }
                }
                if packet.data.len() > 16 {
                    print!("...");
                }
                println!("\n");
            }
            Err(e) => {
                if device.device_type == zigbee_drivers::registry::DeviceType::PcapFile {
                    // End of PCAP file
                    println!("End of PCAP file reached.");
                    break;
                } else {
                    eprintln!("Capture error: {}", e);
                    break;
                }
            }
        }
    }
    
    // Cleanup
    driver.shutdown().await?;
    println!("\nCapture stopped. Total packets: {}", packet_count);
    
    Ok(())
}