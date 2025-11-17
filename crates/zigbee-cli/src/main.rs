use anyhow::Result;
use clap::{Parser, Subcommand};
use zigbee_drivers::registry::DriverRegistry;
use zigbee_analysis::{TopologyMap, TrafficStatistics, ChannelAnalyzer, SecurityAnalyzer};
use std::time::Duration;

#[derive(Parser)]
#[command(name = "zigbee-cli")]
#[command(about = "Zigbee Network Analyzer - Command Line Interface", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List available Zigbee capture devices
    List,
    
    /// Capture packets from the network
    Capture {
        /// Zigbee channel to monitor (11-26)
        #[arg(short, long, default_value_t = 11)]
        channel: u8,
        
        /// Number of packets to capture (0 = infinite)
        #[arg(short = 'n', long, default_value_t = 0)]
        count: u64,
        
        /// Output file for captured packets (JSON format)
        #[arg(short, long)]
        output: Option<String>,
        
        /// Display statistics every N seconds
        #[arg(short, long, default_value_t = 5)]
        stats_interval: u64,
    },
    
    /// Scan all Zigbee channels and show activity
    Scan {
        /// Duration to scan each channel (seconds)
        #[arg(short, long, default_value_t = 2)]
        duration: u64,
    },
    
    /// Analyze channel quality and interference
    Channels {
        /// Duration to analyze (seconds)
        #[arg(short, long, default_value_t = 30)]
        duration: u64,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::List => list_devices().await?,
        Commands::Capture { channel, count, output, stats_interval } => {
            capture_packets(channel, count, output, stats_interval).await?
        },
        Commands::Scan { duration } => scan_channels(duration).await?,
        Commands::Channels { duration } => analyze_channels(duration).await?,
    }

    Ok(())
}

async fn list_devices() -> Result<()> {
    println!("🔍 Scanning for Zigbee devices...\n");
    
    let registry = DriverRegistry::new();
    let devices = registry.detect_devices();
    
    if devices.is_empty() {
        println!("❌ No Zigbee devices found!");
        println!("\nSupported devices:");
        println!("  - Texas Instruments CC2531");
        println!("  - Texas Instruments CC2652");
        println!("  - ConBee/ConBee II");
        return Ok(());
    }
    
    println!("Found {} device(s):\n", devices.len());
    
    for (i, device) in devices.iter().enumerate() {
        println!("  [{}] {} ({})", i, device.driver_name, device.port_name);
        println!("      Type: {:?}", device.device_type);
        println!("      Vendor: 0x{:04x}", 0x0000);  // TODO: get from device
        println!("      Product: 0x{:04x}", 0x0000);  // TODO: get from device
        println!();
    }
    
    Ok(())
}

async fn capture_packets(
    channel: u8,
    count: u64,
    output: Option<String>,
    stats_interval: u64,
) -> Result<()> {
    // Validate channel
    if !(11..=26).contains(&channel) {
        anyhow::bail!("Invalid channel {}. Must be between 11 and 26.", channel);
    }
    
    println!("🎯 Initializing Zigbee capture...\n");
    
    // Find device
    let registry = DriverRegistry::new();
    let devices = registry.detect_devices();
    
    if devices.is_empty() {
        anyhow::bail!("No Zigbee devices found!");
    }
    
    println!("Using device: {} ({})", devices[0].driver_name, devices[0].port_name);
    
    // Initialize driver
    let mut driver = registry.create_driver(&devices[0])
        .ok_or_else(|| anyhow::anyhow!("Failed to create driver"))?;
    
    driver.initialize().await?;
    driver.set_channel(channel).await?;
    
    println!("📡 Capturing on channel {} (2.4 GHz)", channel);
    
    if count > 0 {
        println!("📊 Capturing {} packets...\n", count);
    } else {
        println!("📊 Capturing packets (Ctrl+C to stop)...\n");
    }
    
    // Statistics
    let mut statistics = TrafficStatistics::new();
    let mut security = SecurityAnalyzer::new();
    let mut topology = TopologyMap::new();
    
    let mut packet_count = 0u64;
    let mut last_stats = std::time::Instant::now();
    
    // Packet storage removed until serialization is implemented
    
    loop {
        match driver.capture_packet().await {
            Ok(packet) => {
                packet_count += 1;
                
                // Process packet
                statistics.process_raw_packet(&packet.data, packet.rssi, packet.lqi, packet.channel);
                
                // Try to parse
                if let Ok(parsed) = packet.parse() {
                    topology.process_packet(&parsed);
                    statistics.process_parsed_packet(&parsed, packet.rssi, packet.lqi, packet.channel);
                    security.process_packet(&parsed);
                    
                    // TODO: Add serialization support for output files
                }
                
                // Print packet info
                println!("[{}] RSSI: {:3} dBm | LQI: {:3} | Size: {:3} bytes", 
                    packet_count, packet.rssi, packet.lqi, packet.data.len());
                
                // Print statistics
                if last_stats.elapsed() >= Duration::from_secs(stats_interval) {
                    print_statistics(&statistics, &topology, &security);
                    last_stats = std::time::Instant::now();
                }
                
                // Check count limit
                if count > 0 && packet_count >= count {
                    break;
                }
            }
            Err(e) => {
                eprintln!("⚠️  Capture error: {:?}", e);
            }
        }
    }
    
    println!("\n✅ Capture complete!");
    print_statistics(&statistics, &topology, &security);
    
    // TODO: Save to file if requested (needs Serialize implementation)
    if let Some(output_path) = output {
        println!("\n⚠️  File output not yet implemented (ParsedPacket needs Serialize)");
        println!("    Requested: {}", output_path);
    }
    
    Ok(())
}

async fn scan_channels(duration: u64) -> Result<()> {
    println!("🔍 Scanning all Zigbee channels...\n");
    
    // Find device
    let registry = DriverRegistry::new();
    let devices = registry.detect_devices();
    
    if devices.is_empty() {
        anyhow::bail!("No Zigbee devices found!");
    }
    
    let mut driver = registry.create_driver(&devices[0])
        .ok_or_else(|| anyhow::anyhow!("Failed to create driver"))?;
    
    driver.initialize().await?;
    
    println!("Channel | Packets | Avg RSSI | Devices");
    println!("--------|---------|----------|--------");
    
    for channel in 11..=26 {
        driver.set_channel(channel).await?;
        
        let mut packet_count = 0u64;
        let mut total_rssi = 0i32;
        let mut devices_seen = std::collections::HashSet::new();
        
        let start = std::time::Instant::now();
        
        while start.elapsed() < Duration::from_secs(duration) {
            match tokio::time::timeout(Duration::from_millis(100), driver.capture_packet()).await {
                Ok(Ok(packet)) => {
                    packet_count += 1;
                    total_rssi += packet.rssi as i32;
                    
                    if let Ok(parsed) = packet.parse() {
                        devices_seen.insert(parsed.mac.src_addr);
                        devices_seen.insert(parsed.mac.dest_addr);
                    }
                }
                _ => {}
            }
        }
        
        let avg_rssi = if packet_count > 0 {
            total_rssi / packet_count as i32
        } else {
            0
        };
        
        println!("  {:2}    | {:7} | {:4} dBm | {:7}", 
            channel, packet_count, avg_rssi, devices_seen.len());
    }
    
    println!("\n✅ Scan complete!");
    
    Ok(())
}

async fn analyze_channels(duration: u64) -> Result<()> {
    println!("📊 Analyzing channel quality for {} seconds...\n", duration);
    
    // Find device
    let registry = DriverRegistry::new();
    let devices = registry.detect_devices();
    
    if devices.is_empty() {
        anyhow::bail!("No Zigbee devices found!");
    }
    
    let mut driver = registry.create_driver(&devices[0])
        .ok_or_else(|| anyhow::anyhow!("Failed to create driver"))?;
    
    driver.initialize().await?;
    
    let mut analyzer = ChannelAnalyzer::new();
    let start = std::time::Instant::now();
    
    // Scan all channels
    while start.elapsed() < Duration::from_secs(duration) {
        for channel in 11..=26 {
            driver.set_channel(channel).await?;
            
            for _ in 0..10 {
                if let Ok(packet) = tokio::time::timeout(
                    Duration::from_millis(10),
                    driver.capture_packet()
                ).await {
                    if let Ok(p) = packet {
                        analyzer.process_packet(p.channel, p.rssi, p.lqi, p.data.len(), false);
                    }
                }
            }
        }
    }
    
    println!("Channel | Utilization | Avg RSSI | Packets | Quality");
    println!("--------|-------------|----------|---------|--------");
    
    let channels: Vec<_> = (11..=26).collect();
    
    // Sort by best quality
    if let Some(best) = analyzer.get_best_channel() {
        println!("\n🏆 Best channel: {}\n", best);
    }
    
    for channel in channels {
        if let Some(metrics) = analyzer.get_channel_metrics(channel) {
            let quality = if metrics.utilization < 0.3 {
                "Good"
            } else if metrics.utilization < 0.7 {
                "Fair"
            } else {
                "Poor"
            };
            
            println!("  {:2}    | {:6.1}%     | {:4.0} dBm | {:7} | {}", 
                channel, 
                metrics.utilization * 100.0,
                metrics.avg_rssi,
                metrics.packet_count,
                quality
            );
        }
    }
    
    println!("\n✅ Analysis complete!");
    
    Ok(())
}

fn print_statistics(
    statistics: &TrafficStatistics,
    topology: &TopologyMap,
    security: &SecurityAnalyzer,
) {
    let stats = statistics.get_summary();
    let topo_stats = topology.get_statistics();
    let sec_stats = security.get_statistics();
    
    println!("\n{}", "=".repeat(60));
    println!("📊 STATISTICS");
    println!("{}", "=".repeat(60));
    
    println!("\n📦 Traffic:");
    println!("  Total Packets:     {}", stats.total_packets);
    println!("  Total Bytes:       {:.2} KB", stats.total_bytes as f32 / 1024.0);
    println!("  Packets/sec:       {:.1}", stats.packets_per_second);
    println!("  Avg Packet Size:   {:.1} bytes", stats.avg_packet_size);
    
    println!("\n🌐 Network:");
    println!("  Devices:           {}", topo_stats.total_devices);
    println!("  Networks:          {}", topo_stats.total_networks);
    println!("  Links:             {}", topo_stats.total_links);
    
    println!("\n🔒 Security:");
    println!("  Threats:           {}", sec_stats.total_threats);
    println!("  Encrypted Devices: {}", sec_stats.encrypted_devices);
    println!("  Unencrypted:       {}", sec_stats.unencrypted_devices);
    
    println!("{}\n", "=".repeat(60));
}
