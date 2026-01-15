//! Ethereum chain adapter

use crate::traits::{Chain, ChainFamily, AddressType, GeneratedAddress};
use omnivanity_crypto::{Secp256k1Keypair, hash::keccak256, encoding::eip55_checksum, hex};

/// Ethereum chain
pub struct Ethereum;

impl Chain for Ethereum {
    fn ticker(&self) -> &'static str {
        "ETH"
    }

    fn name(&self) -> &'static str {
        "Ethereum"
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

impl Ethereum {
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
            chain: self.ticker().to_string(),
            address_type: AddressType::Evm,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eth_generation() {
        let eth = Ethereum;
        let addr = eth.generate(AddressType::Evm);
        
        assert!(addr.address.starts_with("0x"));
        assert_eq!(addr.address.len(), 42);
        assert!(addr.private_key_hex.starts_with("0x"));
    }

    #[test]
    fn test_known_vector() {
        let eth = Ethereum;
        // Private key = 1
        let privkey = hex::decode("0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        let addr = eth.generate_from_bytes(&privkey, AddressType::Evm).unwrap();
        
        // Known address for privkey=1
        assert_eq!(addr.address.to_lowercase(), "0x7e5f4552091a69125d5dfcb7b8c2659029395bdf");
    }
}
