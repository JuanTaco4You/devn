//! Stellar chain adapter
//!
//! Stellar StrKey: Ed25519 pubkey + version byte + CRC16 checksum, Base32 encoded

use crate::traits::{Chain, ChainFamily, AddressType, GeneratedAddress};
use omnivanity_crypto::{Ed25519Keypair, hex};

/// Stellar chain
pub struct Stellar;

// CRC16-XModem for Stellar StrKey
fn crc16_xmodem(data: &[u8]) -> u16 {
    let mut crc: u16 = 0x0000;
    for byte in data {
        crc ^= (*byte as u16) << 8;
        for _ in 0..8 {
            if crc & 0x8000 != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}

// Stellar Base32 alphabet (RFC 4648)
const STELLAR_ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

fn base32_encode(data: &[u8]) -> String {
    let mut result = String::new();
    let mut bits = 0u32;
    let mut value = 0u32;
    
    for &byte in data {
        value = (value << 8) | (byte as u32);
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            result.push(STELLAR_ALPHABET[((value >> bits) & 0x1F) as usize] as char);
        }
    }
    
    if bits > 0 {
        result.push(STELLAR_ALPHABET[((value << (5 - bits)) & 0x1F) as usize] as char);
    }
    
    result
}

fn stellar_strkey_encode(version: u8, payload: &[u8]) -> String {
    let mut data = Vec::with_capacity(1 + payload.len() + 2);
    data.push(version);
    data.extend_from_slice(payload);
    
    let checksum = crc16_xmodem(&data);
    data.push((checksum & 0xFF) as u8);
    data.push((checksum >> 8) as u8);
    
    base32_encode(&data)
}

impl Chain for Stellar {
    fn ticker(&self) -> &'static str {
        "XLM"
    }

    fn name(&self) -> &'static str {
        "Stellar"
    }

    fn family(&self) -> ChainFamily {
        ChainFamily::Ed25519
    }

    fn address_types(&self) -> Vec<AddressType> {
        vec![AddressType::Stellar]
    }

    fn default_address_type(&self) -> AddressType {
        AddressType::Stellar
    }

    fn generate(&self, address_type: AddressType) -> GeneratedAddress {
        let keypair = Ed25519Keypair::generate();
        self.generate_from_keypair(&keypair, address_type)
    }

    fn generate_from_bytes(&self, private_key: &[u8], address_type: AddressType) -> Option<GeneratedAddress> {
        if private_key.len() != 32 {
            return None;
        }
        let mut pk = [0u8; 32];
        pk.copy_from_slice(private_key);
        let keypair = Ed25519Keypair::from_bytes(&pk).ok()?;
        Some(self.generate_from_keypair(&keypair, address_type))
    }

    fn valid_address_chars(&self, _address_type: AddressType) -> &'static str {
        "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567"
    }

    fn address_prefix(&self, _address_type: AddressType) -> &'static str {
        "G"
    }
}

impl Stellar {
    fn generate_from_keypair(&self, keypair: &Ed25519Keypair, _address_type: AddressType) -> GeneratedAddress {
        let private_key = keypair.private_key_bytes();
        let public_key = keypair.public_key_bytes();
        
        // Stellar public key address: version byte 0x30 (48) = 'G' prefix
        let address = stellar_strkey_encode(6 << 3, &public_key); // Account ID version
        
        // Stellar secret key: version byte 0x90 (144) = 'S' prefix
        let secret = stellar_strkey_encode(18 << 3, &private_key);
        
        GeneratedAddress {
            address,
            private_key_hex: hex::encode(private_key),
            private_key_native: secret,
            public_key_hex: hex::encode(public_key),
            chain: "XLM".to_string(),
            address_type: AddressType::Stellar,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stellar_generation() {
        let xlm = Stellar;
        let addr = xlm.generate(AddressType::Stellar);
        assert!(addr.address.starts_with("G"));
        assert!(addr.private_key_native.starts_with("S"));
        assert_eq!(addr.chain, "XLM");
    }
}
