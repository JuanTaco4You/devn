//! TON (The Open Network) chain adapter
//!
//! TON address: base64url encoded with CRC16 checksum

use crate::traits::{Chain, ChainFamily, AddressType, GeneratedAddress};
use omnivanity_crypto::{Ed25519Keypair, hex};

/// TON chain
pub struct Ton;

// CRC16-CCITT for TON
fn crc16_ccitt(data: &[u8]) -> u16 {
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

// Base64url encode
fn base64url_encode(data: &[u8]) -> String {
    use std::collections::HashMap;
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    
    let mut result = String::new();
    let mut bits = 0u32;
    let mut value = 0u32;
    
    for &byte in data {
        value = (value << 8) | (byte as u32);
        bits += 8;
        while bits >= 6 {
            bits -= 6;
            result.push(ALPHABET[((value >> bits) & 0x3F) as usize] as char);
        }
    }
    
    if bits > 0 {
        result.push(ALPHABET[((value << (6 - bits)) & 0x3F) as usize] as char);
    }
    
    result
}

impl Chain for Ton {
    fn ticker(&self) -> &'static str {
        "TON"
    }

    fn name(&self) -> &'static str {
        "TON"
    }

    fn family(&self) -> ChainFamily {
        ChainFamily::Ed25519
    }

    fn address_types(&self) -> Vec<AddressType> {
        vec![AddressType::Ton]
    }

    fn default_address_type(&self) -> AddressType {
        AddressType::Ton
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
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_"
    }

    fn address_prefix(&self, _address_type: AddressType) -> &'static str {
        "EQ"
    }
}

impl Ton {
    fn generate_from_keypair(&self, keypair: &Ed25519Keypair, _address_type: AddressType) -> GeneratedAddress {
        let private_key = keypair.private_key_bytes();
        let public_key = keypair.public_key_bytes();
        
        // TON user-friendly address format:
        // [flags(1)] [workchain(1)] [account_id(32)] [crc16(2)]
        // For bounceable mainnet: flags = 0x11, workchain = 0x00
        let mut data = Vec::with_capacity(36);
        data.push(0x11); // Bounceable, mainnet
        data.push(0x00); // Workchain 0
        data.extend_from_slice(&public_key); // Account ID (simplified: using pubkey directly)
        
        let crc = crc16_ccitt(&data);
        data.push((crc >> 8) as u8);
        data.push((crc & 0xFF) as u8);
        
        let address = base64url_encode(&data);
        
        GeneratedAddress {
            address,
            private_key_hex: hex::encode(private_key),
            private_key_native: hex::encode(private_key),
            public_key_hex: hex::encode(public_key),
            chain: "TON".to_string(),
            address_type: AddressType::Ton,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ton_generation() {
        let ton = Ton;
        let addr = ton.generate(AddressType::Ton);
        assert!(addr.address.starts_with("EQ"));
        assert_eq!(addr.chain, "TON");
    }
}
