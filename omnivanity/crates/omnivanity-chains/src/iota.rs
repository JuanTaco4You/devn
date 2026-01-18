//! IOTA chain adapter
//!
//! IOTA Stardust addresses: Blake2b-256(flag || pubkey) = 32 bytes, hex encoded

use crate::traits::{Chain, ChainFamily, AddressType, GeneratedAddress};
use omnivanity_crypto::{Ed25519Keypair, hash::blake2b_256, hex};

/// IOTA chain
pub struct Iota;

impl Chain for Iota {
    fn ticker(&self) -> &'static str {
        "IOTA"
    }

    fn name(&self) -> &'static str {
        "IOTA"
    }

    fn family(&self) -> ChainFamily {
        ChainFamily::Ed25519
    }

    fn address_types(&self) -> Vec<AddressType> {
        vec![AddressType::Iota]
    }

    fn default_address_type(&self) -> AddressType {
        AddressType::Iota
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

impl Iota {
    fn generate_from_keypair(&self, keypair: &Ed25519Keypair, _address_type: AddressType) -> GeneratedAddress {
        let private_key = keypair.private_key_bytes();
        let public_key = keypair.public_key_bytes();
        
        // IOTA Stardust: Blake2b-256(0x00 || pubkey) for Ed25519
        let mut data = Vec::with_capacity(33);
        data.push(0x00); // Ed25519 flag
        data.extend_from_slice(&public_key);
        
        let hash = blake2b_256(&data);
        let address = format!("0x{}", hex::encode(hash));
        
        GeneratedAddress {
            address,
            private_key_hex: hex::encode(private_key),
            private_key_native: hex::encode(private_key),
            public_key_hex: hex::encode(public_key),
            chain: "IOTA".to_string(),
            address_type: AddressType::Iota,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iota_generation() {
        let iota = Iota;
        let addr = iota.generate(AddressType::Iota);
        assert!(addr.address.starts_with("0x"));
        assert_eq!(addr.address.len(), 66);
        assert_eq!(addr.chain, "IOTA");
    }
}
