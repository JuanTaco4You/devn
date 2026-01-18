//! NEAR chain adapter
//!
//! NEAR implicit accounts: 64-character lowercase hex of Ed25519 pubkey

use crate::traits::{Chain, ChainFamily, AddressType, GeneratedAddress};
use omnivanity_crypto::{Ed25519Keypair, hex};

/// NEAR Protocol chain
pub struct Near;

impl Chain for Near {
    fn ticker(&self) -> &'static str {
        "NEAR"
    }

    fn name(&self) -> &'static str {
        "NEAR Protocol"
    }

    fn family(&self) -> ChainFamily {
        ChainFamily::Ed25519
    }

    fn address_types(&self) -> Vec<AddressType> {
        vec![AddressType::Near]
    }

    fn default_address_type(&self) -> AddressType {
        AddressType::Near
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
        ""
    }
}

impl Near {
    fn generate_from_keypair(&self, keypair: &Ed25519Keypair, _address_type: AddressType) -> GeneratedAddress {
        let private_key = keypair.private_key_bytes();
        let public_key = keypair.public_key_bytes();
        
        // NEAR implicit account: lowercase hex of 32-byte Ed25519 pubkey
        let address = hex::encode(public_key);
        
        GeneratedAddress {
            address,
            private_key_hex: hex::encode(private_key),
            private_key_native: format!("ed25519:{}", bs58::encode(&private_key).into_string()),
            public_key_hex: hex::encode(public_key),
            chain: "NEAR".to_string(),
            address_type: AddressType::Near,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_near_generation() {
        let near = Near;
        let addr = near.generate(AddressType::Near);
        assert_eq!(addr.address.len(), 64); // 64 hex chars
        assert_eq!(addr.chain, "NEAR");
    }
}
