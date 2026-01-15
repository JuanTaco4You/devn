//! Solana chain adapter

use crate::traits::{Chain, ChainFamily, AddressType, GeneratedAddress};
use omnivanity_crypto::{Ed25519Keypair, encoding::base58_encode, hex};

/// Solana chain
pub struct Solana;

impl Chain for Solana {
    fn ticker(&self) -> &'static str {
        "SOL"
    }

    fn name(&self) -> &'static str {
        "Solana"
    }

    fn family(&self) -> ChainFamily {
        ChainFamily::Ed25519
    }

    fn address_types(&self) -> Vec<AddressType> {
        vec![AddressType::Solana]
    }

    fn default_address_type(&self) -> AddressType {
        AddressType::Solana
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
        "" // Solana addresses have no fixed prefix
    }
}

impl Solana {
    fn generate_from_keypair(&self, keypair: &Ed25519Keypair, _address_type: AddressType) -> GeneratedAddress {
        let pubkey = keypair.public_key_bytes();
        let address = base58_encode(&pubkey);
        
        // Solana keypair format: 64 bytes (privkey || pubkey)
        let keypair_bytes = keypair.keypair_bytes();
        let private_key_native = format!("[{}]", 
            keypair_bytes.iter()
                .map(|b| b.to_string())
                .collect::<Vec<_>>()
                .join(",")
        );

        GeneratedAddress {
            address,
            private_key_hex: hex::encode(keypair.private_key_bytes()),
            private_key_native,
            public_key_hex: hex::encode(pubkey),
            chain: self.ticker().to_string(),
            address_type: AddressType::Solana,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sol_generation() {
        let sol = Solana;
        let addr = sol.generate(AddressType::Solana);
        
        // Solana addresses are 32-44 chars Base58
        assert!(addr.address.len() >= 32 && addr.address.len() <= 44);
        
        // Should be valid Base58
        for c in addr.address.chars() {
            assert!(sol.valid_address_chars(AddressType::Solana).contains(c));
        }
    }

    #[test]
    fn test_deterministic() {
        let sol = Solana;
        let seed = [42u8; 32];
        
        let addr1 = sol.generate_from_bytes(&seed, AddressType::Solana).unwrap();
        let addr2 = sol.generate_from_bytes(&seed, AddressType::Solana).unwrap();
        
        assert_eq!(addr1.address, addr2.address);
    }
}
