//! Monero (XMR) chain adapter
//!
//! Monero requires dual keypairs (spend/view) and uses unique Base58 encoding.
//! Address: prefix (18) + spend_pub + view_pub + checksum

use crate::traits::{Chain, ChainFamily, AddressType, GeneratedAddress};
use omnivanity_crypto::{
    hex,
    monero::{sc_reduce32, generate_key_image, base58_monero},
    hash::keccak256,
};
use rand::RngCore;

/// Monero chain
pub struct Monero;

impl Chain for Monero {
    fn ticker(&self) -> &'static str {
        "XMR"
    }

    fn name(&self) -> &'static str {
        "Monero"
    }

    fn family(&self) -> ChainFamily {
        // Monero uses Ed25519 on correct curve but with unique scalar reduction
        // so we define it as Ed25519 family generally, but implementation is custom
        ChainFamily::Ed25519
    }

    fn address_types(&self) -> Vec<AddressType> {
        vec![AddressType::Monero] // Will add this type next
    }

    fn default_address_type(&self) -> AddressType {
        AddressType::Monero
    }

    fn generate(&self, address_type: AddressType) -> GeneratedAddress {
        // 1. Generate random seed (32 bytes)
        let mut seed = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut seed);
        
        self.generate_from_bytes(&seed, address_type).unwrap()
    }

    fn generate_from_bytes(&self, private_key: &[u8], _address_type: AddressType) -> Option<GeneratedAddress> {
        if private_key.len() != 32 {
            return None;
        }
        let mut seed = [0u8; 32];
        seed.copy_from_slice(private_key);

        // 2. Reduce seed to get Spend Secret Key
        let spend_secret_scalar = sc_reduce32(&seed);
        let spend_public = generate_key_image(&spend_secret_scalar);
        
        // 3. Hash Spend Secret to get View Secret Key (deterministically)
        // Note: Canonical Monero wallets use keccak256(spend_secret) -> reduced scalar
        let spend_secret_bytes = spend_secret_scalar.to_bytes();
        let view_secret_hash = keccak256(&spend_secret_bytes);
        let view_secret_scalar = sc_reduce32(&view_secret_hash);
        let view_public = generate_key_image(&view_secret_scalar);

        // 4. Construct Address
        // Prefix: 18 (0x12) for primary address
        let network_byte = 18u8;
        
        let mut data = Vec::with_capacity(69); // 1 + 32 + 32 + 4
        data.push(network_byte);
        data.extend_from_slice(&spend_public);
        data.extend_from_slice(&view_public);
        
        // Checksum: First 4 bytes of Keccak256(prefix + spend + view)
        let checksum = keccak256(&data);
        data.extend_from_slice(&checksum[..4]);
        
        // 5. Encode with Monero-specific Base58
        let address = base58_monero::encode(&data);

        Some(GeneratedAddress {
            address,
            private_key_hex: hex::encode(spend_secret_bytes), // Standard seed format
            private_key_native: format!("Spend: {} | View: {}", 
                hex::encode(spend_secret_bytes), 
                hex::encode(view_secret_scalar.to_bytes())
            ),
            public_key_hex: format!("Spend: {} | View: {}", 
                hex::encode(spend_public), 
                hex::encode(view_public)
            ),
            chain: "XMR".to_string(),
            address_type: AddressType::Monero,
        })
    }

    fn valid_address_chars(&self, _address_type: AddressType) -> &'static str {
        "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"
    }

    fn address_prefix(&self, _address_type: AddressType) -> &'static str {
        "4" // Mainnet addresses usually start with 4
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xmr_structure() {
        let xmr = Monero;
        let addr = xmr.generate(AddressType::Monero);
        
        // Standard XMR address is 95 chars
        assert_eq!(addr.address.len(), 95);
        // Starts with 4
        assert!(addr.address.starts_with('4'));
        
        assert_eq!(addr.chain, "XMR");
    }
}
