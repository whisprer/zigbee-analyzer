use thiserror::Error;

#[derive(Error, Debug)]
pub enum HalError {
    #[error("Device not found")]
    DeviceNotFound,
    
    #[error("Device not initialized")]
    NotInitialized,
    
    #[error("Device disconnected")]
    Disconnected,
    
    #[error("Invalid channel: {0} (valid range: 11-26)")]
    InvalidChannel(u8),
    
    #[error("Operation not supported by this device")]
    NotSupported,
    
    #[error("Timeout waiting for packet")]
    Timeout,
    
    #[error("Hardware error: {0}")]
    HardwareError(String),
    
    #[error("USB error: {0}")]
    UsbError(String),
    
    #[error("Serial port error: {0}")]
    SerialError(String),
    
    #[error("Packet buffer full")]
    BufferFull,
    
    #[error("Invalid packet data: {0}")]
    InvalidPacket(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type HalResult<T> = Result<T, HalError>;