//! Tezos chain adapter
//!
//! Tezos uses Base58Check with prefixes: tz1 (Ed25519), tz2 (secp256k1), tz3 (P256)

use crate::traits::{Chain, ChainFamily, AddressType, GeneratedAddress};
use omnivanity_crypto::{Ed25519Keypair, hash::blake2b_160, hex};

/// Tezos chain
pub struct Tezos;

// Tezos-specific Base58Check
fn tezos_base58check_encode(prefix: &[u8], payload: &[u8]) -> String {
    use omnivanity_crypto::hash::double_sha256;
    
    let mut data = Vec::with_capacity(prefix.len() + payload.len() + 4);
    data.extend_from_slice(prefix);
    data.extend_from_slice(payload);
    
    let checksum = double_sha256(&data);
    data.extend_from_slice(&checksum[..4]);
    
    bs58::encode(data).into_string()
}

impl Chain for Tezos {
    fn ticker(&self) -> &'static str {
        "XTZ"
    }

    fn name(&self) -> &'static str {
        "Tezos"
    }

    fn family(&self) -> ChainFamily {
        ChainFamily::Ed25519
    }

    fn address_types(&self) -> Vec<AddressType> {
        vec![AddressType::Tezos]
    }

    fn default_address_type(&self) -> AddressType {
        AddressType::Tezos
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
        "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"
    }

    fn address_prefix(&self, _address_type: AddressType) -> &'static str {
        "tz1"
    }
}

impl Tezos {
    fn generate_from_keypair(&self, keypair: &Ed25519Keypair, _address_type: AddressType) -> GeneratedAddress {
        let private_key = keypair.private_key_bytes();
        let public_key = keypair.public_key_bytes();
        
        // Tezos tz1 address: prefix [6, 161, 159] + Blake2b-160(pubkey)
        let hash = blake2b_160(&public_key);
        let address = tezos_base58check_encode(&[6, 161, 159], &hash);
        
        // Tezos secret key: prefix [43, 246, 78, 7] for edsk (encrypted secret key)
        // but we'll just provide hex for simplicity
        let secret = tezos_base58check_encode(&[43, 246, 78, 7], &private_key);
        
        GeneratedAddress {
            address,
            private_key_hex: hex::encode(private_key),
            private_key_native: secret,
            public_key_hex: hex::encode(public_key),
            chain: "XTZ".to_string(),
            address_type: AddressType::Tezos,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tezos_generation() {
        let xtz = Tezos;
        let addr = xtz.generate(AddressType::Tezos);
        assert!(addr.address.starts_with("tz1"));
        assert_eq!(addr.chain, "XTZ");
    }
}
