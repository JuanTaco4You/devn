//! Midnight chain adapter
//!
//! Midnight uses Bech32m addresses

use crate::traits::{Chain, ChainFamily, AddressType, GeneratedAddress};
use omnivanity_crypto::{Ed25519Keypair, hash::blake2b_256, hex};

/// Midnight chain
pub struct Midnight;

fn midnight_bech32m_encode(data: &[u8]) -> Result<String, String> {
    use bech32::{Bech32m, Hrp};
    let hrp = Hrp::parse("mid").map_err(|e| e.to_string())?;
    bech32::encode::<Bech32m>(hrp, data).map_err(|e| e.to_string())
}

impl Chain for Midnight {
    fn ticker(&self) -> &'static str {
        "NIGHT"
    }

    fn name(&self) -> &'static str {
        "Midnight"
    }

    fn family(&self) -> ChainFamily {
        ChainFamily::Ed25519
    }

    fn address_types(&self) -> Vec<AddressType> {
        vec![AddressType::Midnight]
    }

    fn default_address_type(&self) -> AddressType {
        AddressType::Midnight
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
        "023456789acdefghjklmnpqrstuvwxyz"
    }

    fn address_prefix(&self, _address_type: AddressType) -> &'static str {
        "mid1"
    }
}

impl Midnight {
    fn generate_from_keypair(&self, keypair: &Ed25519Keypair, _address_type: AddressType) -> GeneratedAddress {
        let private_key = keypair.private_key_bytes();
        let public_key = keypair.public_key_bytes();
        
        // Midnight address: bech32m(mid, blake2b-256(pubkey))
        let hash = blake2b_256(&public_key);
        let address = midnight_bech32m_encode(&hash).unwrap_or_default();
        
        GeneratedAddress {
            address,
            private_key_hex: hex::encode(private_key),
            private_key_native: hex::encode(private_key),
            public_key_hex: hex::encode(public_key),
            chain: "NIGHT".to_string(),
            address_type: AddressType::Midnight,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midnight_generation() {
        let night = Midnight;
        let addr = night.generate(AddressType::Midnight);
        assert!(addr.address.starts_with("mid1"));
        assert_eq!(addr.chain, "NIGHT");
    }
}
