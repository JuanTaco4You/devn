//! Internet Computer (ICP) chain adapter
//!
//! ICP Principals are derived from public keys:
//! 1. Hash the DER-encoded public key with SHA-224
//! 2. Append suffix byte 0x02 (self-authenticating)
//! 3. Encode with Base32 without padding, with hyphens for readability
//!
//! This gives a 29-byte identifier encoded as text like:
//! "aaaaa-aaaaa-aaaaa-aaaaa-aaaaa-aaaaa-aaaaa-a"

use crate::traits::{Chain, ChainFamily, AddressType, GeneratedAddress};
use omnivanity_crypto::{Ed25519Keypair, hex};
use sha2::{Sha224, Digest};

/// Internet Computer Principal derivation
pub struct Icp;

// Custom Base32 encoding for ICP (lowercase, no padding)
const ICP_BASE32_ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz234567";

fn base32_encode_icp(data: &[u8]) -> String {
    let mut result = String::new();
    let mut buffer: u64 = 0;
    let mut bits_left = 0;

    for &byte in data {
        buffer = (buffer << 8) | (byte as u64);
        bits_left += 8;

        while bits_left >= 5 {
            bits_left -= 5;
            let idx = ((buffer >> bits_left) & 0x1F) as usize;
            result.push(ICP_BASE32_ALPHABET[idx] as char);
        }
    }

    // Handle remaining bits
    if bits_left > 0 {
        let idx = ((buffer << (5 - bits_left)) & 0x1F) as usize;
        result.push(ICP_BASE32_ALPHABET[idx] as char);
    }

    result
}

fn format_principal(encoded: &str) -> String {
    // Insert hyphens every 5 characters for readability
    encoded
        .chars()
        .collect::<Vec<_>>()
        .chunks(5)
        .map(|c| c.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join("-")
}

impl Icp {
    fn generate_from_keypair(&self, keypair: &Ed25519Keypair) -> GeneratedAddress {
        // Get DER-encoded public key for Ed25519
        // DER header for Ed25519: 30 2a 30 05 06 03 2b 65 70 03 21 00 + 32-byte pubkey
        let der_header: [u8; 12] = [0x30, 0x2a, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x03, 0x21, 0x00];
        let pubkey = keypair.public_key_bytes();
        
        let mut der_encoded = Vec::with_capacity(44);
        der_encoded.extend_from_slice(&der_header);
        der_encoded.extend_from_slice(&pubkey);
        
        // SHA-224 hash of DER-encoded public key
        let mut hasher = Sha224::new();
        Digest::update(&mut hasher, &der_encoded);
        let hash = hasher.finalize();
        
        // Append self-authenticating suffix 0x02
        let mut principal_bytes = Vec::with_capacity(29);
        principal_bytes.extend_from_slice(&hash);
        principal_bytes.push(0x02);
        
        // Base32 encode and format with hyphens
        let encoded = base32_encode_icp(&principal_bytes);
        let address = format_principal(&encoded);
        
        GeneratedAddress {
            address,
            private_key_hex: hex::encode(keypair.private_key_bytes()),
            private_key_native: hex::encode(keypair.private_key_bytes()),
            public_key_hex: hex::encode(pubkey),
            chain: "ICP".to_string(),
            address_type: AddressType::Icp,
        }
    }
}

impl Chain for Icp {
    fn ticker(&self) -> &'static str {
        "ICP"
    }

    fn name(&self) -> &'static str {
        "Internet Computer"
    }

    fn family(&self) -> ChainFamily {
        ChainFamily::Ed25519
    }

    fn address_types(&self) -> Vec<AddressType> {
        vec![AddressType::Icp]
    }

    fn default_address_type(&self) -> AddressType {
        AddressType::Icp
    }

    fn generate(&self, _address_type: AddressType) -> GeneratedAddress {
        let keypair = Ed25519Keypair::generate();
        self.generate_from_keypair(&keypair)
    }

    fn generate_from_bytes(&self, private_key: &[u8], _address_type: AddressType) -> Option<GeneratedAddress> {
        if private_key.len() != 32 {
            return None;
        }
        let mut pk = [0u8; 32];
        pk.copy_from_slice(private_key);
        let keypair = Ed25519Keypair::from_bytes(&pk).ok()?;
        Some(self.generate_from_keypair(&keypair))
    }

    fn valid_address_chars(&self, _address_type: AddressType) -> &'static str {
        "abcdefghijklmnopqrstuvwxyz234567-"
    }

    fn address_prefix(&self, _address_type: AddressType) -> &'static str {
        "" // Principals have no fixed prefix
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icp_generation() {
        let icp = Icp;
        let addr = icp.generate(AddressType::Icp);
        assert!(addr.address.contains('-'));
        assert_eq!(addr.chain, "ICP");
    }
}
