use aes::Aes128;
use ccm::{
    Ccm,
    aead::{Aead, KeyInit, Payload},
    consts::{U4, U8, U13, U16},
};

// Concrete CCM type aliases
type AesCcm32 = Ccm<Aes128, U4, U13>;
type AesCcm64 = Ccm<Aes128, U8, U13>;
type AesCcm128 = Ccm<Aes128, U16, U13>;

/// Security material for Zigbee network
#[derive(Debug, Clone)]
pub struct SecurityMaterial {
    pub network_key: Option<[u8; 16]>,
    pub link_keys: Vec<LinkKey>,
    pub trust_center_address: Option<[u8; 8]>,
    pub key_sequence_number: u8,
}

#[derive(Debug, Clone)]
pub struct LinkKey {
    pub device_address: [u8; 8],
    pub key: [u8; 16],
}

impl SecurityMaterial {
    pub fn new() -> Self {
        Self {
            network_key: None,
            link_keys: Vec::new(),
            trust_center_address: None,
            key_sequence_number: 0,
        }
    }
    
    pub fn set_network_key(&mut self, key: [u8; 16]) {
        self.network_key = Some(key);
    }
    
    pub fn add_link_key(&mut self, device_address: [u8; 8], key: [u8; 16]) {
        self.link_keys.retain(|lk| lk.device_address != device_address);
        self.link_keys.push(LinkKey { device_address, key });
    }
    
    pub fn get_link_key(&self, device_address: &[u8; 8]) -> Option<&[u8; 16]> {
        self.link_keys
            .iter()
            .find(|lk| &lk.device_address == device_address)
            .map(|lk| &lk.key)
    }
}

impl Default for SecurityMaterial {
    fn default() -> Self {
        Self::new()
    }
}

/// Auxiliary security header
#[derive(Debug, Clone)]
pub struct AuxiliarySecurityHeader {
    pub security_control: SecurityControl,
    pub frame_counter: u32,
    pub source_address: [u8; 8],
    pub key_sequence_number: Option<u8>,
}

/// Security control field
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SecurityControl {
    pub security_level: SecurityLevel,
    pub key_identifier: KeyIdentifier,
    pub extended_nonce: bool,
}

/// Zigbee security levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityLevel {
    None = 0x00,
    Mic32 = 0x01,
    Mic64 = 0x02,
    Mic128 = 0x03,
    EncMic32 = 0x04,
    EncMic64 = 0x05,
    EncMic128 = 0x06,
    Enc = 0x07,
}

/// Key identifier modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyIdentifier {
    DataKey = 0x00,
    NetworkKey = 0x01,
    KeyTransportKey = 0x02,
    KeyLoadKey = 0x03,
}

impl SecurityLevel {
    pub fn mic_length(&self) -> usize {
        match self {
            SecurityLevel::None | SecurityLevel::Enc => 0,
            SecurityLevel::Mic32 | SecurityLevel::EncMic32 => 4,
            SecurityLevel::Mic64 | SecurityLevel::EncMic64 => 8,
            SecurityLevel::Mic128 | SecurityLevel::EncMic128 => 16,
        }
    }
    
    pub fn uses_encryption(&self) -> bool {
        matches!(
            self,
            SecurityLevel::Enc
                | SecurityLevel::EncMic32
                | SecurityLevel::EncMic64
                | SecurityLevel::EncMic128
        )
    }
    
    pub fn uses_mic(&self) -> bool {
        !matches!(self, SecurityLevel::None | SecurityLevel::Enc)
    }
}

impl SecurityControl {
    pub fn to_byte(&self) -> u8 {
        let mut byte = self.security_level as u8;
        byte |= (self.key_identifier as u8) << 3;
        if self.extended_nonce {
            byte |= 0x20;
        }
        byte
    }
}

/// Construct nonce for AES-CCM*
fn construct_nonce(
    source_address: &[u8; 8],
    frame_counter: u32,
    security_control: u8,
) -> [u8; 13] {
    let mut nonce = [0u8; 13];
    nonce[0..8].copy_from_slice(source_address);
    nonce[8..12].copy_from_slice(&frame_counter.to_le_bytes());
    nonce[12] = security_control;
    nonce
}

/// Decrypt a secured frame using AES-CCM*
pub fn decrypt_frame(
    encrypted_data: &[u8],
    aux_header: &AuxiliarySecurityHeader,
    key: &[u8; 16],
    additional_data: &[u8],
) -> Result<Vec<u8>, String> {
    let security_level = aux_header.security_control.security_level;
    
    if !security_level.uses_encryption() && !security_level.uses_mic() {
        return Ok(encrypted_data.to_vec());
    }
    
    let mic_len = security_level.mic_length();
    
    if encrypted_data.len() < mic_len {
        return Err("Encrypted data too short for MIC".to_string());
    }
    
    let payload_len = encrypted_data.len() - mic_len;
    let ciphertext = &encrypted_data[..payload_len];
    let mic = &encrypted_data[payload_len..];
    
    let nonce = construct_nonce(
        &aux_header.source_address,
        aux_header.frame_counter,
        aux_header.security_control.to_byte(),
    );
    
    match mic_len {
        0 => decrypt_ctr_mode(ciphertext, key, &nonce),
        4 => decrypt_ccm_32(ciphertext, mic, key, &nonce, additional_data),
        8 => decrypt_ccm_64(ciphertext, mic, key, &nonce, additional_data),
        16 => decrypt_ccm_128(ciphertext, mic, key, &nonce, additional_data),
        _ => Err("Invalid MIC length".to_string()),
    }
}

fn decrypt_ccm_32(
    ciphertext: &[u8],
    mic: &[u8],
    key: &[u8; 16],
    nonce: &[u8; 13],
    additional_data: &[u8],
) -> Result<Vec<u8>, String> {
    let cipher = AesCcm32::new(key.into());
    
    let mut combined = ciphertext.to_vec();
    combined.extend_from_slice(mic);
    
    let payload = Payload {
        msg: &combined,
        aad: additional_data,
    };
    
    cipher
        .decrypt(nonce.into(), payload)
        .map_err(|_| "Decryption failed".to_string())
}

fn decrypt_ccm_64(
    ciphertext: &[u8],
    mic: &[u8],
    key: &[u8; 16],
    nonce: &[u8; 13],
    additional_data: &[u8],
) -> Result<Vec<u8>, String> {
    let cipher = AesCcm64::new(key.into());
    
    let mut combined = ciphertext.to_vec();
    combined.extend_from_slice(mic);
    
    let payload = Payload {
        msg: &combined,
        aad: additional_data,
    };
    
    cipher
        .decrypt(nonce.into(), payload)
        .map_err(|_| "Decryption failed".to_string())
}

fn decrypt_ccm_128(
    ciphertext: &[u8],
    mic: &[u8],
    key: &[u8; 16],
    nonce: &[u8; 13],
    additional_data: &[u8],
) -> Result<Vec<u8>, String> {
    let cipher = AesCcm128::new(key.into());
    
    let mut combined = ciphertext.to_vec();
    combined.extend_from_slice(mic);
    
    let payload = Payload {
        msg: &combined,
        aad: additional_data,
    };
    
    cipher
        .decrypt(nonce.into(), payload)
        .map_err(|_| "Decryption failed".to_string())
}

/// Decrypt using CTR mode only
fn decrypt_ctr_mode(
    ciphertext: &[u8],
    key: &[u8; 16],
    nonce: &[u8; 13],
) -> Result<Vec<u8>, String> {
    use aes::cipher::{BlockEncrypt, KeyInit as _};
    
    let cipher = Aes128::new(key.into());
    let mut plaintext = Vec::with_capacity(ciphertext.len());
    
    let mut counter = [0u8; 16];
    counter[0..13].copy_from_slice(nonce);
    counter[13] = 0;
    
    let mut block_counter = 1u16;
    let mut offset = 0;
    
    while offset < ciphertext.len() {
        counter[14..16].copy_from_slice(&block_counter.to_be_bytes());
        
        let mut keystream_block = counter;
        cipher.encrypt_block((&mut keystream_block).into());
        
        let chunk_len = (ciphertext.len() - offset).min(16);
        for i in 0..chunk_len {
            plaintext.push(ciphertext[offset + i] ^ keystream_block[i]);
        }
        
        offset += chunk_len;
        block_counter += 1;
    }
    
    Ok(plaintext)
}

/// Encrypt a frame using AES-CCM*
pub fn encrypt_frame(
    plaintext: &[u8],
    aux_header: &AuxiliarySecurityHeader,
    key: &[u8; 16],
    additional_data: &[u8],
) -> Result<Vec<u8>, String> {
    let security_level = aux_header.security_control.security_level;
    
    if !security_level.uses_encryption() && !security_level.uses_mic() {
        return Ok(plaintext.to_vec());
    }
    
    let mic_len = security_level.mic_length();
    
    let nonce = construct_nonce(
        &aux_header.source_address,
        aux_header.frame_counter,
        aux_header.security_control.to_byte(),
    );
    
    match mic_len {
        0 => encrypt_ctr_mode(plaintext, key, &nonce),
        4 => encrypt_ccm_32(plaintext, key, &nonce, additional_data),
        8 => encrypt_ccm_64(plaintext, key, &nonce, additional_data),
        16 => encrypt_ccm_128(plaintext, key, &nonce, additional_data),
        _ => Err("Invalid MIC length".to_string()),
    }
}

fn encrypt_ccm_32(
    plaintext: &[u8],
    key: &[u8; 16],
    nonce: &[u8; 13],
    additional_data: &[u8],
) -> Result<Vec<u8>, String> {
    let cipher = AesCcm32::new(key.into());
    
    let payload = Payload {
        msg: plaintext,
        aad: additional_data,
    };
    
    cipher
        .encrypt(nonce.into(), payload)
        .map_err(|_| "Encryption failed".to_string())
}

fn encrypt_ccm_64(
    plaintext: &[u8],
    key: &[u8; 16],
    nonce: &[u8; 13],
    additional_data: &[u8],
) -> Result<Vec<u8>, String> {
    let cipher = AesCcm64::new(key.into());
    
    let payload = Payload {
        msg: plaintext,
        aad: additional_data,
    };
    
    cipher
        .encrypt(nonce.into(), payload)
        .map_err(|_| "Encryption failed".to_string())
}

fn encrypt_ccm_128(
    plaintext: &[u8],
    key: &[u8; 16],
    nonce: &[u8; 13],
    additional_data: &[u8],
) -> Result<Vec<u8>, String> {
    let cipher = AesCcm128::new(key.into());
    
    let payload = Payload {
        msg: plaintext,
        aad: additional_data,
    };
    
    cipher
        .encrypt(nonce.into(), payload)
        .map_err(|_| "Encryption failed".to_string())
}

/// Encrypt using CTR mode only
fn encrypt_ctr_mode(
    plaintext: &[u8],
    key: &[u8; 16],
    nonce: &[u8; 13],
) -> Result<Vec<u8>, String> {
    decrypt_ctr_mode(plaintext, key, nonce)
}