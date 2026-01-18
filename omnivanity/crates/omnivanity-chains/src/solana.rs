//! Solana chain family
//!
//! Covers: SOL and Solana-based tokens (TRUMP, BONK, PENGU, JUP, PUMP)
//! All use: Ed25519 pubkey as base58 address

use crate::traits::{Chain, ChainFamily, AddressType, GeneratedAddress};
use omnivanity_crypto::{Ed25519Keypair, encoding::base58_encode, hex};

/// Solana-style chain with configurable ticker/name
#[derive(Debug, Clone, Copy)]
pub struct SolanaChain {
    ticker: &'static str,
    name: &'static str,
}

impl SolanaChain {
    pub const fn new(ticker: &'static str, name: &'static str) -> Self {
        Self { ticker, name }
    }

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
            chain: self.ticker.to_string(),
            address_type: AddressType::Solana,
        }
    }
}

// Native Solana
pub const SOL: SolanaChain = SolanaChain::new("SOL", "Solana");

// Solana Tokens (Memes)
pub const TRUMP: SolanaChain = SolanaChain::new("TRUMP", "OFFICIAL TRUMP");
pub const BONK: SolanaChain = SolanaChain::new("BONK", "Bonk");
pub const PENGU: SolanaChain = SolanaChain::new("PENGU", "Pudgy Penguins");
pub const PUMP: SolanaChain = SolanaChain::new("PUMP", "Pump.fun");

// Solana Tokens (DeFi)
pub const JUP: SolanaChain = SolanaChain::new("JUP", "Jupiter");
pub const RENDER: SolanaChain = SolanaChain::new("RENDER", "Render");

// Legacy alias
pub type Solana = SolanaChain;
pub const SOLANA: SolanaChain = SOL;

impl Chain for SolanaChain {
    fn ticker(&self) -> &'static str {
        self.ticker
    }

    fn name(&self) -> &'static str {
        self.name
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sol_generation() {
        let addr = SOL.generate(AddressType::Solana);
        assert!(addr.address.len() >= 32 && addr.address.len() <= 44);
        assert_eq!(addr.chain, "SOL");
    }

    #[test]
    fn test_trump_generation() {
        let addr = TRUMP.generate(AddressType::Solana);
        assert!(addr.address.len() >= 32 && addr.address.len() <= 44);
        assert_eq!(addr.chain, "TRUMP");
    }
}
