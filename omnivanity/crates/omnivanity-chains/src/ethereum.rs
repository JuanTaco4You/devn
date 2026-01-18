//! EVM chain family
//!
//! Covers: ETH, BNB, MATIC, ARB, OP, AVAX, FTM, GNO, CELO, etc.
//! All use: secp256k1 + Keccak-256(pubkey[1..65]) last 20 bytes + EIP-55 checksum

use crate::traits::{Chain, ChainFamily, AddressType, GeneratedAddress};
use omnivanity_crypto::{Secp256k1Keypair, hash::keccak256, encoding::eip55_checksum, hex};

/// EVM-compatible chain with configurable ticker/name
#[derive(Debug, Clone, Copy)]
pub struct EvmChain {
    ticker: &'static str,
    name: &'static str,
}

impl EvmChain {
    pub const fn new(ticker: &'static str, name: &'static str) -> Self {
        Self { ticker, name }
    }

    fn generate_from_keypair(&self, keypair: &Secp256k1Keypair, _address_type: AddressType) -> GeneratedAddress {
        // ETH address = last 20 bytes of keccak256(uncompressed_pubkey[1..65])
        let pubkey_xy = keypair.public_key_xy();
        let hash = keccak256(&pubkey_xy);
        
        let mut address_bytes = [0u8; 20];
        address_bytes.copy_from_slice(&hash[12..32]);
        
        let address = eip55_checksum(&address_bytes);
        let private_key = keypair.private_key_bytes();
        
        GeneratedAddress {
            address,
            private_key_hex: format!("0x{}", hex::encode(private_key)),
            private_key_native: format!("0x{}", hex::encode(private_key)),
            public_key_hex: format!("0x{}", hex::encode(keypair.public_key_uncompressed())),
            chain: self.ticker.to_string(),
            address_type: AddressType::Evm,
        }
    }
}

// Pre-defined EVM chains
pub const ETH: EvmChain = EvmChain::new("ETH", "Ethereum");
pub const BNB: EvmChain = EvmChain::new("BNB", "BNB Smart Chain");
pub const MATIC: EvmChain = EvmChain::new("MATIC", "Polygon");
pub const ARB: EvmChain = EvmChain::new("ARB", "Arbitrum");
pub const OP: EvmChain = EvmChain::new("OP", "Optimism");
pub const AVAX: EvmChain = EvmChain::new("AVAX", "Avalanche C-Chain");
pub const FTM: EvmChain = EvmChain::new("FTM", "Fantom");
pub const GNO: EvmChain = EvmChain::new("GNO", "Gnosis Chain");
pub const CELO: EvmChain = EvmChain::new("CELO", "Celo");

// Keep legacy Ethereum struct for backwards compatibility
pub type Ethereum = EvmChain;
pub const ETHEREUM: EvmChain = ETH;

impl Chain for EvmChain {
    fn ticker(&self) -> &'static str {
        self.ticker
    }

    fn name(&self) -> &'static str {
        self.name
    }

    fn family(&self) -> ChainFamily {
        ChainFamily::Evm
    }

    fn address_types(&self) -> Vec<AddressType> {
        vec![AddressType::Evm]
    }

    fn default_address_type(&self) -> AddressType {
        AddressType::Evm
    }

    fn generate(&self, address_type: AddressType) -> GeneratedAddress {
        let keypair = Secp256k1Keypair::generate();
        self.generate_from_keypair(&keypair, address_type)
    }

    fn generate_from_bytes(&self, private_key: &[u8], address_type: AddressType) -> Option<GeneratedAddress> {
        if private_key.len() != 32 {
            return None;
        }
        let mut pk = [0u8; 32];
        pk.copy_from_slice(private_key);
        let keypair = Secp256k1Keypair::from_bytes(&pk).ok()?;
        Some(self.generate_from_keypair(&keypair, address_type))
    }

    fn valid_address_chars(&self, _address_type: AddressType) -> &'static str {
        "0123456789abcdefABCDEF"
    }

    fn address_prefix(&self, _address_type: AddressType) -> &'static str {
        "0x"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eth_generation() {
        let addr = ETH.generate(AddressType::Evm);
        assert!(addr.address.starts_with("0x"));
        assert_eq!(addr.address.len(), 42);
        assert_eq!(addr.chain, "ETH");
    }

    #[test]
    fn test_bnb_generation() {
        let addr = BNB.generate(AddressType::Evm);
        assert!(addr.address.starts_with("0x"));
        assert_eq!(addr.chain, "BNB");
    }

    #[test]
    fn test_matic_generation() {
        let addr = MATIC.generate(AddressType::Evm);
        assert!(addr.address.starts_with("0x"));
        assert_eq!(addr.chain, "MATIC");
    }

    #[test]
    fn test_known_vector() {
        // Private key = 1
        let privkey = hex::decode("0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        let addr = ETH.generate_from_bytes(&privkey, AddressType::Evm).unwrap();
        
        // Known address for privkey=1
        assert_eq!(addr.address.to_lowercase(), "0x7e5f4552091a69125d5dfcb7b8c2659029395bdf");
    }
}
