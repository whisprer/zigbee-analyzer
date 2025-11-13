use zigbee_drivers::DriverRegistry;
use std::env;

fn main() {
    println!("Zigbee Analyzer - Device Detection");
    println!("===================================\n");
    
    let mut registry = DriverRegistry::new();
    
    // Check command line for PCAP files to add
    let args: Vec<String> = env::args().collect();
    for arg in args.iter().skip(1) {
        if arg.ends_with(".pcap") || arg.ends_with(".pcapng") || arg.ends_with(".cap") {
            match registry.add_pcap_file(arg) {
                Ok(_) => println!("Added PCAP file: {}", arg),
                Err(e) => eprintln!("Failed to add {}: {}", arg, e),
            }
        }
    }
    
    // List supported hardware
    println!("\nSupported Hardware:");
    println!("-------------------");
    for driver in registry.list_drivers() {
        println!("  {} - {} ({:04x}:{:04x})", 
            driver.name, 
            driver.description, 
            driver.vid, 
            driver.pid
        );
    }
    
    // Detect connected hardware
    println!("\nDetected Hardware Devices:");
    println!("--------------------------");
    let hardware = registry.detect_hardware();
    if hardware.is_empty() {
        println!("  (none)");
    } else {
        for device in &hardware {
            println!("  {}", device);
        }
    }
    
    // List PCAP files
    println!("\nRegistered PCAP Files:");
    println!("----------------------");
    let pcap_devices = registry.get_pcap_devices();
    if pcap_devices.is_empty() {
        println!("  (none)");
    } else {
        for device in &pcap_devices {
            println!("  {}", device);
        }
    }
    
    // Show all devices
    println!("\nAll Available Devices:");
    println!("----------------------");
    let all_devices = registry.detect_devices();
    if all_devices.is_empty() {
        println!("  (none)");
    } else {
        for (i, device) in all_devices.iter().enumerate() {
            println!("  {}. {} [{}]", 
                i + 1, 
                device,
                match device.device_type {
                    zigbee_drivers::registry::DeviceType::Hardware => "Hardware",
                    zigbee_drivers::registry::DeviceType::PcapFile => "File",
                }
            );
        }
    }
    
    println!("\nUsage: {} [pcap_file1] [pcap_file2] ...", 
        args.get(0).unwrap_or(&"list_devices".to_string()));
}