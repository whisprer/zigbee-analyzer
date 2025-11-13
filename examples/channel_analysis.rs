use zigbee_drivers::DriverRegistry;
use zigbee_hal::traits::ZigbeeCapture;
use zigbee_analysis::ChannelAnalyzer;
use std::env;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Zigbee Analyzer - Channel Analysis");
    println!("===================================\n");
    
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
    
    let mut analyzer = ChannelAnalyzer::new();
    
    println!("Analyzing channels...");
    println!("Collecting data for 30 seconds\n");
    
    let start = std::time::Instant::now();
    let mut packet_count = 0;
    
    while start.elapsed() < Duration::from_secs(30) {
        match tokio::time::timeout(Duration::from_millis(100), driver.capture_packet()).await {
            Ok(Ok(packet)) => {
                packet_count += 1;
                
                let has_errors = !packet.validate_fcs();
                analyzer.process_packet(packet.channel, packet.rssi, packet.lqi, packet.data.len(), has_errors);
                
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
    
    println!("\n\n=== Channel Analysis Results ===\n");
    
    // Show all channels
    println!("Channel Quality Scores:");
    println!("----------------------");
    for (channel, score) in analyzer.get_channels_by_quality() {
        if let Some(metrics) = analyzer.get_channel_metrics(channel) {
            println!("Channel {:2} ({:4} MHz): {:.1}/100 - RSSI: {:5.1} dBm, LQI: {:5.1}, Util: {:4.1}%, Pkts: {}",
                channel,
                metrics.frequency_mhz,
                score,
                metrics.avg_rssi,
                metrics.avg_lqi,
                metrics.utilization * 100.0,
                metrics.packet_count
            );
            
            if metrics.interference_score > 0.3 {
                println!("         ⚠ Interference detected: {:?} (score: {:.2})",
                    metrics.interference_type,
                    metrics.interference_score
                );
            }
        }
    }
    
    println!();
    
    // Best/worst channels
    if let Some(best) = analyzer.get_best_channel() {
        println!("Best Channel: {}", best);
    }
    if let Some(worst) = analyzer.get_worst_channel() {
        println!("Worst Channel: {}", worst);
    }
    
    println!();
    
    // Channel recommendation
    let recommendation = analyzer.recommend_channel();
    println!("=== Channel Recommendation ===");
    println!("Recommended: Channel {} (score: {:.1}/100)",
        recommendation.recommended_channel,
        recommendation.quality_score
    );
    println!("Reason: {}", recommendation.reason);
    
    if !recommendation.alternatives.is_empty() {
        println!("\nAlternatives:");
        for (i, (channel, score)) in recommendation.alternatives.iter().enumerate() {
            println!("  {}. Channel {} (score: {:.1}/100)", i + 1, channel, score);
        }
    }
    
    println!();
    
    // Interference detection
    let interfered = analyzer.get_channels_with_interference(0.3);
    if !interfered.is_empty() {
        println!("=== Channels with Interference ===");
        for (channel, itype, score) in interfered {
            println!("Channel {}: {:?} (severity: {:.2})", channel, itype, score);
        }
        println!();
    }
    
    // WiFi interference
    let wifi_channels = analyzer.detect_wifi_interference();
    if !wifi_channels.is_empty() {
        println!("=== WiFi Interference Detected ===");
        println!("Affected Zigbee channels: {:?}", wifi_channels);
        println!("Recommendation: Use Zigbee channels that don't overlap with WiFi");
        println!("  WiFi Ch 1  → Avoid Zigbee 11-14");
        println!("  WiFi Ch 6  → Avoid Zigbee 16-19");
        println!("  WiFi Ch 11 → Avoid Zigbee 21-24");
        println!();
    }
    
    // Detailed metrics for top 5 channels
    println!("=== Detailed Channel Metrics (Top 5) ===");
    for (channel, _) in analyzer.get_channels_by_quality().iter().take(5) {
        if let Some(metrics) = analyzer.get_channel_metrics(*channel) {
            println!("\nChannel {} ({} MHz):", channel, metrics.frequency_mhz);
            println!("  Quality Score: {:.1}/100", metrics.quality_score);
            println!("  RSSI: avg={:.1} dBm, min={} dBm, max={} dBm",
                metrics.avg_rssi, metrics.min_rssi, metrics.max_rssi);
            println!("  LQI: avg={:.1}, min={}, max={}",
                metrics.avg_lqi, metrics.min_lqi, metrics.max_lqi);
            println!("  Traffic: {} packets ({} bytes)", metrics.packet_count, metrics.byte_count);
            println!("  Rate: {:.1} pps", metrics.packets_per_second);
            println!("  Utilization: {:.1}%", metrics.utilization * 100.0);
            println!("  Error Rate: {:.2}%", metrics.error_rate * 100.0);
            println!("  Noise Floor: {} dBm", metrics.noise_floor);
            
            if metrics.interference_score > 0.0 {
                println!("  Interference: {:?} (score: {:.2})",
                    metrics.interference_type, metrics.interference_score);
            }
        }
    }
    
    // Generate spectrum visualization data
    println!("\n=== Spectrum Overview ===");
    let spectrum = analyzer.get_spectrum_data();
    println!("Channel utilization map:");
    print!("Ch:  ");
    for ch_data in &spectrum.channels {
        print!("{:2} ", ch_data.channel);
    }
    println!();
    print!("Util:");
    for ch_data in &spectrum.channels {
        let util = (ch_data.utilization * 10.0) as usize;
        let bar = "█".repeat(util) + &"░".repeat(10 - util);
        print!("{} ", if util > 0 { &bar[..1] } else { " " });
    }
    println!();
    
    driver.shutdown().await?;
    
    Ok(())
}