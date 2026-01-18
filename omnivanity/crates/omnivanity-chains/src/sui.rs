//! Sui chain adapter
//!
//! Sui address: Blake2b-256(flag || pubkey) = 32 bytes, hex encoded

use crate::traits::{Chain, ChainFamily, AddressType, GeneratedAddress};
use omnivanity_crypto::{Ed25519Keypair, hash::blake2b_256, hex};

/// Sui chain
pub struct Sui;

impl Chain for Sui {
    fn ticker(&self) -> &'static str {
        "SUI"
    }

    fn name(&self) -> &'static str {
        "Sui"
    }

    fn family(&self) -> ChainFamily {
        ChainFamily::Ed25519
    }

    fn address_types(&self) -> Vec<AddressType> {
        vec![AddressType::Sui]
    }

    fn default_address_type(&self) -> AddressType {
        AddressType::Sui
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
        "0123456789abcdef"
    }

    fn address_prefix(&self, _address_type: AddressType) -> &'static str {
        "0x"
    }
}

impl Sui {
    fn generate_from_keypair(&self, keypair: &Ed25519Keypair, _address_type: AddressType) -> GeneratedAddress {
        let private_key = keypair.private_key_bytes();
        let public_key = keypair.public_key_bytes();
        
        // Sui address: Blake2b-256(0x00 || pubkey) for Ed25519
        let mut data = Vec::with_capacity(33);
        data.push(0x00); // Ed25519 flag byte
        data.extend_from_slice(&public_key);
        
        let hash = blake2b_256(&data);
        let address = format!("0x{}", hex::encode(hash));
        
        GeneratedAddress {
            address,
            private_key_hex: hex::encode(private_key),
            private_key_native: hex::encode(private_key),
            public_key_hex: hex::encode(public_key),
            chain: "SUI".to_string(),
            address_type: AddressType::Sui,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sui_generation() {
        let sui = Sui;
        let addr = sui.generate(AddressType::Sui);
        assert!(addr.address.starts_with("0x"));
        assert_eq!(addr.address.len(), 66); // 0x + 64 hex chars
        assert_eq!(addr.chain, "SUI");
    }
}
