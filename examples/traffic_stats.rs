use zigbee_drivers::DriverRegistry;
use zigbee_hal::traits::ZigbeeCapture;
use zigbee_analysis::TrafficStatistics;
use std::env;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Zigbee Analyzer - Traffic Statistics");
    println!("=====================================\n");
    
    let args: Vec<String> = env::args().collect();
    
    // Setup registry
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
    
    let mut stats = TrafficStatistics::new();
    
    println!("Collecting statistics...");
    println!("Press Ctrl+C to stop\n");
    
    let start = std::time::Instant::now();
    let mut last_print = std::time::Instant::now();
    
    loop {
        match tokio::time::timeout(Duration::from_millis(100), driver.capture_packet()).await {
            Ok(Ok(packet)) => {
                // Parse packet
                match packet.parse() {
                    Ok(parsed) => {
                        stats.process_parsed_packet(&parsed, packet.rssi, packet.lqi, packet.channel);
                    }
                    Err(_) => {
                        stats.process_raw_packet(&packet.data, packet.rssi, packet.lqi, packet.channel);
                        stats.record_parse_error();
                    }
                }
                
                // Check FCS
                if !packet.validate_fcs() {
                    stats.record_fcs_error();
                }
            }
            Ok(Err(_)) => {
                break;
            }
            Err(_) => {
                // Timeout
            }
        }
        
        // Print stats every 5 seconds
        if last_print.elapsed() >= Duration::from_secs(5) {
            print_stats(&stats);
            last_print = std::time::Instant::now();
        }
        
        // Stop after 60 seconds for non-hardware sources
        if start.elapsed() >= Duration::from_secs(60) {
            break;
        }
    }
    
    println!("\n\n=== Final Statistics ===\n");
    print_detailed_stats(&stats);
    
    driver.shutdown().await?;
    
    Ok(())
}

fn print_stats(stats: &TrafficStatistics) {
    let summary = stats.get_summary();
    
    println!("\r\x1B[2J\x1B[H"); // Clear screen
    println!("=== Live Statistics ===");
    println!("Uptime: {}s", summary.uptime_seconds);
    println!("Total Packets: {} ({:.2} KB)", summary.total_packets, summary.total_bytes as f32 / 1024.0);
    println!("Average Packet Size: {:.1} bytes", summary.avg_packet_size);
    println!();
    println!("Current Rate: {:.1} pps, {:.1} bps", summary.packets_per_second, summary.bytes_per_second);
    println!("Peak Rate: {:.1} pps, {:.1} bps", summary.peak_pps, summary.peak_bps);
    println!();
    println!("Frame Types:");
    println!("  Beacon: {}", summary.beacon_frames);
    println!("  Data: {}", summary.data_frames);
    println!("  ACK: {}", summary.ack_frames);
    println!("  Command: {}", summary.command_frames);
    println!();
    println!("Unique Devices: {}", summary.unique_devices);
    println!("Active Channels: {}", summary.active_channels);
    println!();
    println!("Errors:");
    println!("  Parse Errors: {}", summary.parse_errors);
    println!("  FCS Errors: {}", summary.fcs_errors);
}

fn print_detailed_stats(stats: &TrafficStatistics) {
    let summary = stats.get_summary();
    
    println!("Total Packets: {}", summary.total_packets);
    println!("Total Bytes: {} ({:.2} MB)", summary.total_bytes, summary.total_bytes as f32 / 1_048_576.0);
    println!("Average Packet Size: {:.1} bytes", summary.avg_packet_size);
    println!("Uptime: {}s", summary.uptime_seconds);
    println!();
    
    println!("Frame Type Distribution:");
    let total = summary.beacon_frames + summary.data_frames + summary.ack_frames + summary.command_frames;
    if total > 0 {
        println!("  Beacon:  {:6} ({:5.1}%)", summary.beacon_frames, summary.beacon_frames as f32 / total as f32 * 100.0);
        println!("  Data:    {:6} ({:5.1}%)", summary.data_frames, summary.data_frames as f32 / total as f32 * 100.0);
        println!("  ACK:     {:6} ({:5.1}%)", summary.ack_frames, summary.ack_frames as f32 / total as f32 * 100.0);
        println!("  Command: {:6} ({:5.1}%)", summary.command_frames, summary.command_frames as f32 / total as f32 * 100.0);
    }
    println!();
    
    println!("Channel Statistics:");
    let mut channels: Vec<_> = stats.channel_stats.iter().collect();
    channels.sort_by_key(|(ch, _)| **ch);
    for (channel, ch_stats) in channels {
        println!("  Channel {}: {} packets, Avg RSSI: {:.1} dBm, Avg LQI: {:.1}, Util: {:.1}%",
            channel,
            ch_stats.packet_count,
            ch_stats.avg_rssi,
            ch_stats.avg_lqi,
            ch_stats.utilization * 100.0
        );
    }
    println!();
    
    println!("Top 10 Devices:");
    for (i, (addr, dev_stats)) in stats.get_top_devices(10).iter().enumerate() {
        println!("  {}. {} - TX: {}, RX: {}, RSSI: {:.1} dBm",
            i + 1,
            addr,
            dev_stats.tx_packets,
            dev_stats.rx_packets,
            dev_stats.avg_rssi
        );
    }
    println!();
    
    println!("Top 10 Profiles:");
    for (i, (profile, count)) in stats.get_top_profiles(10).iter().enumerate() {
        println!("  {}. 0x{:04x}: {} packets", i + 1, profile, count);
    }
    println!();
    
    println!("Top 10 Clusters:");
    for (i, (cluster, count)) in stats.get_top_clusters(10).iter().enumerate() {
        println!("  {}. 0x{:04x}: {} packets", i + 1, cluster, count);
    }
    println!();
    
    println!("Protocol Statistics:");
    println!("  MAC with security: {}", stats.protocol_stats.mac_with_security);
    println!("  MAC with ACK request: {}", stats.protocol_stats.mac_with_ack_request);
    println!("  Network layer packets: {}", stats.protocol_stats.nwk_packets);
    println!("  APS layer packets: {}", stats.protocol_stats.aps_packets);
    println!("  ZCL packets: {}", stats.protocol_stats.zcl_packets);
}