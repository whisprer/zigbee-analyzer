use zigbee_drivers::PcapReader;
use zigbee_hal::traits::ZigbeeCapture;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get PCAP file from command line
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <pcap_file>", args[0]);
        eprintln!("\nExample: {} capture.pcap", args[0]);
        return Ok(());
    }
    
    let pcap_file = &args[1];
    
    println!("Zigbee Analyzer - PCAP Analysis Example");
    println!("=========================================\n");
    println!("Loading PCAP file: {}\n", pcap_file);
    
    // Create PCAP reader
    let mut reader = PcapReader::new(pcap_file)?;
    
    // Optionally set playback speed (0.0 = as fast as possible)
    reader.set_playback_speed(0.0);
    
    // Initialize (loads the file)
    reader.initialize().await?;
    
    println!("Loaded {} packets\n", reader.packet_count());
    
    // Analyze packets
    let mut packet_count = 0;
    let mut beacon_count = 0;
    let mut data_count = 0;
    let mut ack_count = 0;
    
    loop {
        match reader.capture_packet().await {
            Ok(packet) => {
                packet_count += 1;
                
                // Parse packet
                if let Ok(parsed) = packet.parse() {
                    use zigbee_core::FrameType;
                    
                    match parsed.mac.frame_control.frame_type {
                        FrameType::Beacon => beacon_count += 1,
                        FrameType::Data => data_count += 1,
                        FrameType::Acknowledgment => ack_count += 1,
                        _ => {}
                    }
                    
                    if packet_count <= 10 {
                        println!("Packet #{}", packet_count);
                        println!("  Type: {:?}", parsed.mac.frame_control.frame_type);
                        println!("  Src: {}", parsed.mac.src_addr);
                        println!("  Dst: {}", parsed.mac.dest_addr);
                        println!("  RSSI: {} dBm", packet.rssi);
                        
                        if let Some(nwk) = parsed.network {
                            println!("  NWK Src: 0x{:04x}", nwk.src_addr);
                            println!("  NWK Dst: 0x{:04x}", nwk.dest_addr);
                        }
                        println!();
                    }
                }
            }
            Err(_) => {
                // End of file
                break;
            }
        }
    }
    
    println!("\nAnalysis Complete");
    println!("=================");
    println!("Total packets: {}", packet_count);
    println!("  Beacon frames: {}", beacon_count);
    println!("  Data frames: {}", data_count);
    println!("  ACK frames: {}", ack_count);
    println!("  Other: {}", packet_count - beacon_count - data_count - ack_count);
    
    Ok(())
}