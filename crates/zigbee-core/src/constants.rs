/// IEEE 802.15.4 channel definitions for 2.4 GHz band
pub const CHANNEL_MIN: u8 = 11;
pub const CHANNEL_MAX: u8 = 26;
pub const CHANNEL_COUNT: usize = 16;

/// Channel to frequency mapping (in MHz)
pub fn channel_to_frequency(channel: u8) -> Option<u16> {
    if (CHANNEL_MIN..=CHANNEL_MAX).contains(&channel) {
		Some(2405 + ((channel - 11) * 5) as u16)		
    } else {
        None
    }
}

/// Common PAN IDs
pub const PAN_ID_BROADCAST: u16 = 0xffff;

/// Common short addresses
pub const SHORT_ADDR_BROADCAST: u16 = 0xffff;
pub const SHORT_ADDR_COORDINATOR: u16 = 0x0000;
pub const SHORT_ADDR_NONE: u16 = 0xfffe;

/// Maximum frame sizes
pub const MAX_PHY_PACKET_SIZE: usize = 127;
pub const MAX_MAC_PAYLOAD_SIZE: usize = 102;

/// Common endpoints
pub const ENDPOINT_ZDO: u8 = 0x00;
pub const ENDPOINT_BROADCAST: u8 = 0xff;

/// Security levels
pub const SECURITY_LEVEL_NONE: u8 = 0;
pub const SECURITY_LEVEL_MIC32: u8 = 1;
pub const SECURITY_LEVEL_MIC64: u8 = 2;
pub const SECURITY_LEVEL_MIC128: u8 = 3;
pub const SECURITY_LEVEL_ENC: u8 = 4;
pub const SECURITY_LEVEL_ENC_MIC32: u8 = 5;
pub const SECURITY_LEVEL_ENC_MIC64: u8 = 6;
pub const SECURITY_LEVEL_ENC_MIC128: u8 = 7;