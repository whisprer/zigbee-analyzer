use zigbee_drivers::{DriverRegistry, PcapWriter};
use zigbee_hal::traits::ZigbeeCapture;
use std::env;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <output_file.pcap> [duration_seconds]", args[0]);
        return Ok(());
    }
    
    let output_file = &args[1];
    let duration_secs: u64 = args.get(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(60);
    
    println!("Zigbee Analyzer - PCAP Recording Example");
    println!("==========================================\n");
    
    // Detect device
    let registry = DriverRegistry::new();
    let devices = registry.detect_devices();
    
    if devices.is_empty() {
        eprintln!("No Zigbee devices found!");
        return Ok(());
    }
    
    println!("Using device: {}", devices[0]);
    
    // Create driver
    let mut driver = registry.create_driver(&devices[0])
        .ok_or("Failed to create driver")?;
    
    driver.initialize().await?;
    driver.set_channel(11).await?;
    
    // Create PCAP writer
    let mut writer = PcapWriter::new(output_file, false)?;
    writer.open()?;
    
    println!("Recording to: {}", output_file);
    println!("Duration: {} seconds", duration_secs);
    println!("Press Ctrl+C to stop early\n");
    
    let start = std::time::Instant::now();
    let mut packet_count = 0;
    
    while start.elapsed() < Duration::from_secs(duration_secs) {
        match tokio::time::timeout(Duration::from_secs(1), driver.capture_packet()).await {
            Ok(Ok(packet)) => {
                writer.write_packet(&packet)?;
                packet_count += 1;
                
                if packet_count % 100 == 0 {
                    println!("Captured {} packets...", packet_count);
                }
            }
            Ok(Err(e)) => {
                eprintln!("Capture error: {}", e);
                break;
            }
            Err(_) => {
                // Timeout, continue
            }
        }
    }
    
    writer.close()?;
    driver.shutdown().await?;
    
    println!("\nRecording complete!");
    println!("Captured {} packets to {}", packet_count, output_file);
    
    Ok(())
}