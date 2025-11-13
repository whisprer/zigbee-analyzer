use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use zigbee_drivers::DriverRegistry;
use zigbee_hal::traits::ZigbeeCapture;
use zigbee_analysis::{
    NetworkTopology, TrafficStatistics, ChannelAnalyzer, AnomalyDetector,
    SecurityAnalyzer, DeviceDatabase,
};
use std::time::{SystemTime, Duration};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Overview,
    Topology,
    Statistics,
    Channels,
    Anomalies,
    Security,
    Devices,
}

pub struct App {
    // UI state
    pub current_tab: Tab,
    pub should_quit: bool,
    pub paused: bool,
    pub help_visible: bool,
    
    // Driver
    driver_rx: mpsc::UnboundedReceiver<CaptureEvent>,
    
    // Analyzers
    pub topology: NetworkTopology,
    pub statistics: TrafficStatistics,
    pub channels: ChannelAnalyzer,
    pub anomalies: AnomalyDetector,
    pub security: SecurityAnalyzer,
    pub devices: DeviceDatabase,
    
    // State
    pub start_time: SystemTime,
    pub packet_count: u64,
    pub error_count: u64,
    
    // Scroll state
    pub scroll_offset: usize,
}

enum CaptureEvent {
    Packet(zigbee_hal::ZigbeePacket),
    Error(String),
}

impl App {
    pub async fn new() -> Result<Self> {
        // Initialize driver
        let mut registry = DriverRegistry::new();
        let devices = registry.detect_devices();
        
        if devices.is_empty() {
            anyhow::bail!("No Zigbee devices found!");
        }
        
        let mut driver = registry.create_driver(&devices[0])
            .ok_or_else(|| anyhow::anyhow!("Failed to create driver"))?;
        
        driver.initialize().await?;
        let _ = driver.set_channel(11).await;
        
        // Create channel for packet events
        let (tx, rx) = mpsc::unbounded_channel();
        
        // Spawn capture task
        tokio::spawn(async move {
            loop {
                match driver.capture_packet().await {
                    Ok(packet) => {
                        if tx.send(CaptureEvent::Packet(packet)).is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(CaptureEvent::Error(format!("{:?}", e)));
                    }
                }
            }
        });
        
        Ok(Self {
            current_tab: Tab::Overview,
            should_quit: false,
            paused: false,
            help_visible: false,
            driver_rx: rx,
            topology: NetworkTopology::new(),
            statistics: TrafficStatistics::new(),
            channels: ChannelAnalyzer::new(),
            anomalies: AnomalyDetector::new(),
            security: SecurityAnalyzer::new(),
            devices: DeviceDatabase::new(),
            start_time: SystemTime::now(),
            packet_count: 0,
            error_count: 0,
            scroll_offset: 0,
        })
    }
    
    pub async fn on_tick(&mut self) -> Result<()> {
        if self.paused {
            return Ok(());
        }
        
        // Process available packets (non-blocking)
        let mut processed = 0;
        while processed < 100 {
            match self.driver_rx.try_recv() {
                Ok(CaptureEvent::Packet(packet)) => {
                    self.packet_count += 1;
                    
                    // Process raw packet
                    self.statistics.process_raw_packet(
                        &packet.data,
                        packet.rssi,
                        packet.lqi,
                        packet.channel
                    );
                    
                    let has_errors = !packet.validate_fcs();
                    if has_errors {
                        self.error_count += 1;
                    }
                    
                    self.channels.process_packet(
                        packet.channel,
                        packet.rssi,
                        packet.lqi,
                        packet.data.len(),
                        has_errors
                    );
                    
                    // Parse and process
                    if let Ok(parsed) = packet.parse() {
                        self.topology.process_packet(&parsed, packet.rssi, packet.lqi, packet.channel);
                        self.statistics.process_parsed_packet(&parsed, packet.rssi, packet.lqi, packet.channel);
                        self.anomalies.process_packet(&parsed, packet.rssi, packet.channel);
                        self.security.process_packet(&parsed);
                        self.devices.process_packet(&parsed);
                    } else {
                        self.statistics.record_parse_error();
                    }
                    
                    processed += 1;
                }
                Ok(CaptureEvent::Error(_)) => {
                    self.error_count += 1;
                    break;
                }
                Err(_) => break,
            }
        }
        
        Ok(())
    }
    
    pub async fn on_key(&mut self, key: KeyEvent) -> Result<bool> {
        // Global keys
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.should_quit = true;
            return Ok(false);
        }
        
        match key.code {
            KeyCode::Char('q') => {
                self.should_quit = true;
                return Ok(false);
            }
            KeyCode::Char('?') | KeyCode::F(1) => {
                self.help_visible = !self.help_visible;
            }
            KeyCode::Char(' ') => {
                self.paused = !self.paused;
            }
            KeyCode::Char('1') => self.current_tab = Tab::Overview,
            KeyCode::Char('2') => self.current_tab = Tab::Topology,
            KeyCode::Char('3') => self.current_tab = Tab::Statistics,
            KeyCode::Char('4') => self.current_tab = Tab::Channels,
            KeyCode::Char('5') => self.current_tab = Tab::Anomalies,
            KeyCode::Char('6') => self.current_tab = Tab::Security,
            KeyCode::Char('7') => self.current_tab = Tab::Devices,
            KeyCode::Tab => {
                self.current_tab = match self.current_tab {
                    Tab::Overview => Tab::Topology,
                    Tab::Topology => Tab::Statistics,
                    Tab::Statistics => Tab::Channels,
                    Tab::Channels => Tab::Anomalies,
                    Tab::Anomalies => Tab::Security,
                    Tab::Security => Tab::Devices,
                    Tab::Devices => Tab::Overview,
                };
            }
            KeyCode::Up => {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
            }
            KeyCode::Down => {
                self.scroll_offset = self.scroll_offset.saturating_add(1);
            }
            KeyCode::PageUp => {
                self.scroll_offset = self.scroll_offset.saturating_sub(10);
            }
            KeyCode::PageDown => {
                self.scroll_offset = self.scroll_offset.saturating_add(10);
            }
            KeyCode::Home => {
                self.scroll_offset = 0;
            }
            _ => {}
        }
        
        Ok(true)
    }
    
    pub fn uptime(&self) -> Duration {
        SystemTime::now()
            .duration_since(self.start_time)
            .unwrap_or(Duration::from_secs(0))
    }
}