use zigbee_hal::traits::ZigbeeCapture;
use std::fmt;
use std::path::{Path, PathBuf};

/// Information about an available driver
#[derive(Debug, Clone)]
pub struct DriverInfo {
    pub name: &'static str,
    pub description: &'static str,
    pub vid: u16,
    pub pid: u16,
}

/// A detected device (hardware or file)
#[derive(Debug, Clone)]
pub struct DetectedDevice {
    pub driver_name: &'static str,
    pub device_id: String,
    pub port_name: String,
    pub device_type: DeviceType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceType {
    Hardware,
    PcapFile,
}

impl fmt::Display for DetectedDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.device_type {
            DeviceType::Hardware => write!(f, "{} at {}", self.driver_name, self.port_name),
            DeviceType::PcapFile => write!(f, "{}: {}", self.driver_name, self.port_name),
        }
    }
}

/// Registry of all available drivers
pub struct DriverRegistry {
    drivers: Vec<DriverInfo>,
    pcap_files: Vec<PathBuf>,
}

impl DriverRegistry {
    pub fn new() -> Self {
        Self {
            drivers: vec![
                DriverInfo {
                    name: "TI CC2531",
                    description: "Texas Instruments CC2531 USB Dongle",
                    vid: 0x0451,
                    pid: 0x16ae,
                },
                DriverInfo {
                    name: "TI CC2652",
                    description: "Texas Instruments CC2652 USB Dongle",
                    vid: 0x0451,
                    pid: 0x16c8,
                },
                DriverInfo {
                    name: "TI CC2652P",
                    description: "Texas Instruments CC2652P USB Dongle (High Power)",
                    vid: 0x0451,
                    pid: 0x16c9,
                },
                DriverInfo {
                    name: "TI CC2652RB",
                    description: "Texas Instruments CC2652RB USB Dongle",
                    vid: 0x0451,
                    pid: 0x16ca,
                },
                DriverInfo {
                    name: "ConBee",
                    description: "Dresden Elektronik ConBee USB Dongle",
                    vid: 0x1cf1,
                    pid: 0x0030,
                },
                DriverInfo {
                    name: "ConBee II",
                    description: "Dresden Elektronik ConBee II USB Dongle",
                    vid: 0x1cf1,
                    pid: 0x0031,
                },
                DriverInfo {
                    name: "RaspBee",
                    description: "Dresden Elektronik RaspBee Module",
                    vid: 0x1cf1,
                    pid: 0x0028,
                },
                DriverInfo {
                    name: "RaspBee II",
                    description: "Dresden Elektronik RaspBee II Module",
                    vid: 0x1cf1,
                    pid: 0x0029,
                },
            ],
            pcap_files: Vec::new(),
        }
    }
    
    /// Get all registered hardware drivers
    pub fn list_drivers(&self) -> &[DriverInfo] {
        &self.drivers
    }
    
    /// Add a PCAP file to the registry
    pub fn add_pcap_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), String> {
        let path = path.as_ref();
        
        if !path.exists() {
            return Err(format!("File not found: {:?}", path));
        }
        
        // Verify it's a valid PCAP file by checking extension
        match path.extension().and_then(|s| s.to_str()) {
            Some("pcap") | Some("pcapng") | Some("cap") => {
                self.pcap_files.push(path.to_path_buf());
                Ok(())
            }
            _ => Err("File must have .pcap, .pcapng, or .cap extension".to_string()),
        }
    }
    
    /// Remove a PCAP file from the registry
    pub fn remove_pcap_file<P: AsRef<Path>>(&mut self, path: P) {
        let path = path.as_ref();
        self.pcap_files.retain(|p| p != path);
    }
    
    /// Get list of registered PCAP files
    pub fn list_pcap_files(&self) -> &[PathBuf] {
        &self.pcap_files
    }
    
    /// Clear all registered PCAP files
    pub fn clear_pcap_files(&mut self) {
        self.pcap_files.clear();
    }
    
    /// Detect all connected hardware Zigbee devices
    pub fn detect_hardware(&self) -> Vec<DetectedDevice> {
        let mut detected = Vec::new();
        
        if let Ok(ports) = serialport::available_ports() {
            for port in ports {
                if let serialport::SerialPortType::UsbPort(info) = &port.port_type {
                    // Check against registered drivers
                    for driver in &self.drivers {
                        if info.vid == driver.vid && info.pid == driver.pid {
                            detected.push(DetectedDevice {
                                driver_name: driver.name,
                                device_id: format!("{:04x}:{:04x}", info.vid, info.pid),
                                port_name: port.port_name.clone(),
                                device_type: DeviceType::Hardware,
                            });
                        }
                    }
                }
            }
        }
        
        detected
    }
    
    /// Get all PCAP files as DetectedDevice entries
    pub fn get_pcap_devices(&self) -> Vec<DetectedDevice> {
        self.pcap_files
            .iter()
            .map(|path| DetectedDevice {
                driver_name: "PCAP File",
                device_id: format!("pcap:{}", path.display()),
                port_name: path.display().to_string(),
                device_type: DeviceType::PcapFile,
            })
            .collect()
    }
    
    /// Detect all available devices (hardware + PCAP files)
    pub fn detect_devices(&self) -> Vec<DetectedDevice> {
        let mut all_devices = self.detect_hardware();
        all_devices.extend(self.get_pcap_devices());
        all_devices
    }
    
    /// Create a driver instance for a detected device
    pub fn create_driver(&self, device: &DetectedDevice) -> Option<Box<dyn ZigbeeCapture>> {
        match device.device_type {
            DeviceType::Hardware => self.create_hardware_driver(device),
            DeviceType::PcapFile => self.create_pcap_reader(device),
        }
    }
    
    /// Create a hardware driver instance
    fn create_hardware_driver(&self, device: &DetectedDevice) -> Option<Box<dyn ZigbeeCapture>> {
        match device.driver_name {
            "TI CC2531" => {
                crate::ti_cc2531::CC2531::new().ok()
                    .map(|d| Box::new(d) as Box<dyn ZigbeeCapture>)
            }
            "TI CC2652" | "TI CC2652P" | "TI CC2652RB" => {
                crate::ti_cc2652::CC2652::new_with_port(device.port_name.clone()).ok()
                    .map(|d| Box::new(d) as Box<dyn ZigbeeCapture>)
            }
            "ConBee" => {
                crate::conbee::ConBee::new_with_port(
                    device.port_name.clone(),
                    crate::conbee::ConBeeVariant::ConBee
                ).ok()
                    .map(|d| Box::new(d) as Box<dyn ZigbeeCapture>)
            }
            "ConBee II" => {
                crate::conbee::ConBee::new_with_port(
                    device.port_name.clone(),
                    crate::conbee::ConBeeVariant::ConBeeII
                ).ok()
                    .map(|d| Box::new(d) as Box<dyn ZigbeeCapture>)
            }
            "RaspBee" => {
                crate::conbee::ConBee::new_with_port(
                    device.port_name.clone(),
                    crate::conbee::ConBeeVariant::RaspBee
                ).ok()
                    .map(|d| Box::new(d) as Box<dyn ZigbeeCapture>)
            }
            "RaspBee II" => {
                crate::conbee::ConBee::new_with_port(
                    device.port_name.clone(),
                    crate::conbee::ConBeeVariant::RaspBeeII
                ).ok()
                    .map(|d| Box::new(d) as Box<dyn ZigbeeCapture>)
            }
            _ => None,
        }
    }
    
    /// Create a PCAP reader instance
    fn create_pcap_reader(&self, device: &DetectedDevice) -> Option<Box<dyn ZigbeeCapture>> {
        crate::pcap::PcapReader::new(&device.port_name).ok()
            .map(|d| Box::new(d) as Box<dyn ZigbeeCapture>)
    }
    
    /// Create a PCAP reader directly from a path
    pub fn create_pcap_reader_from_path<P: AsRef<Path>>(&self, path: P) -> Option<Box<dyn ZigbeeCapture>> {
        crate::pcap::PcapReader::new(path).ok()
            .map(|d| Box::new(d) as Box<dyn ZigbeeCapture>)
    }
}

impl Default for DriverRegistry {
    fn default() -> Self {
        Self::new()
    }
}