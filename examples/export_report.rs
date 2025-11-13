use zigbee_drivers::DriverRegistry;
use zigbee_hal::traits::ZigbeeCapture;
use zigbee_analysis::{
    NetworkTopology, TrafficStatistics, ChannelAnalyzer, AnomalyDetector,
    SecurityAnalyzer, DeviceDatabase, ExportManager,
};
use std::env;
use std::time::{Duration, SystemTime};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Zigbee Analyzer - Export Report Generator");
    println!("==========================================\n");
    
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
    
    // Initialize all analyzers
    let mut topology = NetworkTopology::new();
    let mut statistics = TrafficStatistics::new();
    let mut channels = ChannelAnalyzer::new();
    let mut anomalies = AnomalyDetector::new();
    let mut security = SecurityAnalyzer::new();
    let mut device_db = DeviceDatabase::new();
    
    println!("Analyzing network...");
    println!("This will take 30 seconds\n");
    
    let start_time = SystemTime::now();
    let start = std::time::Instant::now();
    let mut packet_count = 0;
    
    while start.elapsed() < Duration::from_secs(30) {
        match tokio::time::timeout(Duration::from_millis(100), driver.capture_packet()).await {
            Ok(Ok(packet)) => {
                packet_count += 1;
                
                // Process raw packet
                statistics.process_raw_packet(&packet.data, packet.rssi, packet.lqi, packet.channel);
                channels.process_packet(packet.channel, packet.rssi, packet.lqi, packet.data.len(), false);
                
                // Parse and process
                if let Ok(parsed) = packet.parse() {
                    topology.process_packet(&parsed, packet.rssi, packet.lqi, packet.channel);
                    statistics.process_parsed_packet(&parsed, packet.rssi, packet.lqi, packet.channel);
                    anomalies.process_packet(&parsed, packet.rssi, packet.channel);
                    security.process_packet(&parsed);
                    device_db.process_packet(&parsed);
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
    
    println!("\n\nGenerating reports...\n");
    
    // Create export manager
    let exporter = ExportManager::new();
    
    // Create snapshot
    let snapshot = exporter.create_snapshot(
        Some(&topology),
        Some(&statistics),
        Some(&channels),
        Some(&anomalies),
        Some(&security),
        Some(&device_db),
        start_time,
    );
    
    // Export to JSON
    exporter.export_json(&snapshot, "report.json")?;
    println!("✓ JSON report exported to: report.json");
    
    // Export to HTML
    exporter.export_html_report(&snapshot, "report.html")?;
    println!("✓ HTML report exported to: report.html");
    
    // Export to Markdown
    exporter.export_markdown_report(&snapshot, "report.md")?;
    println!("✓ Markdown report exported to: report.md");
    
    // Export topology CSV
    exporter.export_topology_csv(&topology, "topology.csv")?;
    println!("✓ Topology CSV exported to: topology.csv");
    
    // Export anomalies CSV
    exporter.export_anomalies_csv(&anomalies, "anomalies.csv")?;
    println!("✓ Anomalies CSV exported to: anomalies.csv");
    
    println!("\nAll reports generated successfully!");
    println!("\nReport Summary:");
    println!("  Packets Analyzed: {}", packet_count);
    println!("  Devices Found: {}", topology.devices().len());
    println!("  Anomalies: {}", anomalies.get_anomalies().len());
    println!("  Security Incidents: {}", security.get_incidents().len());
    
    driver.shutdown().await?;
    
    Ok(())
}