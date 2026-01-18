//! Aptos chain adapter
//!
//! Aptos address: SHA3-256(pubkey || signature_scheme_id) = 32 bytes, hex encoded

use crate::traits::{Chain, ChainFamily, AddressType, GeneratedAddress};
use omnivanity_crypto::{Ed25519Keypair, hash::sha3_256, hex};

/// Aptos chain
pub struct Aptos;

impl Chain for Aptos {
    fn ticker(&self) -> &'static str {
        "APT"
    }

    fn name(&self) -> &'static str {
        "Aptos"
    }

    fn family(&self) -> ChainFamily {
        ChainFamily::Ed25519
    }

    fn address_types(&self) -> Vec<AddressType> {
        vec![AddressType::Aptos]
    }

    fn default_address_type(&self) -> AddressType {
        AddressType::Aptos
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

impl Aptos {
    fn generate_from_keypair(&self, keypair: &Ed25519Keypair, _address_type: AddressType) -> GeneratedAddress {
        let private_key = keypair.private_key_bytes();
        let public_key = keypair.public_key_bytes();
        
        // Aptos address: SHA3-256(pubkey || 0x00) for Ed25519 single-key
        let mut data = Vec::with_capacity(33);
        data.extend_from_slice(&public_key);
        data.push(0x00); // Ed25519 scheme identifier
        
        let hash = sha3_256(&data);
        let address = format!("0x{}", hex::encode(hash));
        
        GeneratedAddress {
            address,
            private_key_hex: hex::encode(private_key),
            private_key_native: hex::encode(private_key),
            public_key_hex: hex::encode(public_key),
            chain: "APT".to_string(),
            address_type: AddressType::Aptos,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aptos_generation() {
        let apt = Aptos;
        let addr = apt.generate(AddressType::Aptos);
        assert!(addr.address.starts_with("0x"));
        assert_eq!(addr.address.len(), 66); // 0x + 64 hex chars
        assert_eq!(addr.chain, "APT");
    }
}
