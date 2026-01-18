//! Algorand chain adapter
//!
//! Algorand address: Ed25519 pubkey + 4-byte checksum (last 4 bytes of SHA512/256), Base32 encoded

use crate::traits::{Chain, ChainFamily, AddressType, GeneratedAddress};
use omnivanity_crypto::{Ed25519Keypair, hex};
use sha2::{Sha512_256, Digest};

/// Algorand chain
pub struct Algorand;

// RFC 4648 Base32 alphabet (no padding)
const BASE32_ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

fn base32_encode_algo(data: &[u8]) -> String {
    let mut result = String::new();
    let mut bits = 0u32;
    let mut value = 0u32;
    
    for &byte in data {
        value = (value << 8) | (byte as u32);
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            result.push(BASE32_ALPHABET[((value >> bits) & 0x1F) as usize] as char);
        }
    }
    
    if bits > 0 {
        result.push(BASE32_ALPHABET[((value << (5 - bits)) & 0x1F) as usize] as char);
    }
    
    result
}

impl Chain for Algorand {
    fn ticker(&self) -> &'static str {
        "ALGO"
    }

    fn name(&self) -> &'static str {
        "Algorand"
    }

    fn family(&self) -> ChainFamily {
        ChainFamily::Ed25519
    }

    fn address_types(&self) -> Vec<AddressType> {
        vec![AddressType::Algorand]
    }

    fn default_address_type(&self) -> AddressType {
        AddressType::Algorand
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
        ""
    }
}

impl Algorand {
    fn generate_from_keypair(&self, keypair: &Ed25519Keypair, _address_type: AddressType) -> GeneratedAddress {
        let private_key = keypair.private_key_bytes();
        let public_key = keypair.public_key_bytes();
        
        // Algorand: checksum = last 4 bytes of SHA512/256(pubkey)
        let mut hasher = Sha512_256::new();
        hasher.update(&public_key);
        let hash = hasher.finalize();
        
        // Address = Base32(pubkey || checksum)
        let mut address_data = Vec::with_capacity(36);
        address_data.extend_from_slice(&public_key);
        address_data.extend_from_slice(&hash[28..32]); // Last 4 bytes
        
        let address = base32_encode_algo(&address_data);
        
        GeneratedAddress {
            address,
            private_key_hex: hex::encode(private_key),
            private_key_native: hex::encode(private_key),
            public_key_hex: hex::encode(public_key),
            chain: "ALGO".to_string(),
            address_type: AddressType::Algorand,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_algo_generation() {
        let algo = Algorand;
        let addr = algo.generate(AddressType::Algorand);
        assert_eq!(addr.address.len(), 58); // 36 bytes = 58 base32 chars
        assert_eq!(addr.chain, "ALGO");
    }
}
