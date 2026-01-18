//! Nano chain adapter
//!
//! Nano: Base32 encoded Ed25519 pubkey + Blake2b checksum

use crate::traits::{Chain, ChainFamily, AddressType, GeneratedAddress};
use omnivanity_crypto::{Ed25519Keypair, hash::blake2b_256, hex};

/// Nano chain
pub struct Nano;

// Nano uses a custom Base32 alphabet
const NANO_ALPHABET: &[u8] = b"13456789abcdefghijkmnopqrstuwxyz";

fn nano_base32_encode(data: &[u8]) -> String {
    let mut result = String::new();
    let mut bits = 0u32;
    let mut value = 0u64;
    
    for &byte in data {
        value = (value << 8) | (byte as u64);
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            result.push(NANO_ALPHABET[((value >> bits) & 0x1F) as usize] as char);
        }
    }
    
    if bits > 0 {
        result.push(NANO_ALPHABET[((value << (5 - bits)) & 0x1F) as usize] as char);
    }
    
    result
}

impl Chain for Nano {
    fn ticker(&self) -> &'static str {
        "XNO"
    }

    fn name(&self) -> &'static str {
        "Nano"
    }

    fn family(&self) -> ChainFamily {
        ChainFamily::Ed25519
    }

    fn address_types(&self) -> Vec<AddressType> {
        vec![AddressType::Nano]
    }

    fn default_address_type(&self) -> AddressType {
        AddressType::Nano
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
        "13456789abcdefghijkmnopqrstuwxyz"
    }

    fn address_prefix(&self, _address_type: AddressType) -> &'static str {
        "nano_"
    }
}

impl Nano {
    fn generate_from_keypair(&self, keypair: &Ed25519Keypair, _address_type: AddressType) -> GeneratedAddress {
        let private_key = keypair.private_key_bytes();
        let public_key = keypair.public_key_bytes();
        
        // Nano checksum: last 5 bytes of Blake2b-256(pubkey), reversed
        let hash = blake2b_256(&public_key);
        let mut checksum = [0u8; 5];
        checksum.copy_from_slice(&hash[27..32]);
        checksum.reverse();
        
        // Encode pubkey (52 chars) + checksum (8 chars)
        let pubkey_encoded = nano_base32_encode(&public_key);
        let checksum_encoded = nano_base32_encode(&checksum);
        
        let address = format!("nano_{}{}", pubkey_encoded, checksum_encoded);
        
        GeneratedAddress {
            address,
            private_key_hex: hex::encode(private_key),
            private_key_native: hex::encode(private_key),
            public_key_hex: hex::encode(public_key),
            chain: "XNO".to_string(),
            address_type: AddressType::Nano,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nano_generation() {
        let nano = Nano;
        let addr = nano.generate(AddressType::Nano);
        assert!(addr.address.starts_with("nano_"));
        assert_eq!(addr.chain, "XNO");
    }
}
