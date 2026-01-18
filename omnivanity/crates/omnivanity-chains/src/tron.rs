//! TRON chain adapter
//!
//! TRON address: Keccak256(pubkey) last 20 bytes + 0x41 prefix + Base58Check

use crate::traits::{Chain, ChainFamily, AddressType, GeneratedAddress};
use omnivanity_crypto::{
    Secp256k1Keypair,
    hash::keccak256,
    encoding::base58check_encode,
    hex,
};

/// TRON chain
pub struct Tron;

impl Chain for Tron {
    fn ticker(&self) -> &'static str {
        "TRX"
    }

    fn name(&self) -> &'static str {
        "TRON"
    }

    fn family(&self) -> ChainFamily {
        ChainFamily::Evm // Uses secp256k1 like EVM
    }

    fn address_types(&self) -> Vec<AddressType> {
        vec![AddressType::Tron]
    }

    fn default_address_type(&self) -> AddressType {
        AddressType::Tron
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
        "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"
    }

    fn address_prefix(&self, _address_type: AddressType) -> &'static str {
        "T"
    }
}

impl Tron {
    fn generate_from_keypair(&self, keypair: &Secp256k1Keypair, _address_type: AddressType) -> GeneratedAddress {
        let private_key = keypair.private_key_bytes();
        
        // TRON: Keccak256(uncompressed_pubkey[1..65]) last 20 bytes
        let pubkey_xy = keypair.public_key_xy();
        let hash = keccak256(&pubkey_xy);
        
        let mut address_bytes = [0u8; 20];
        address_bytes.copy_from_slice(&hash[12..32]);
        
        // TRON address: version byte 0x41 (65) + 20-byte payload, Base58Check encoded
        let address = base58check_encode(0x41, &address_bytes);
        
        GeneratedAddress {
            address,
            private_key_hex: hex::encode(private_key),
            private_key_native: hex::encode(private_key),
            public_key_hex: hex::encode(keypair.public_key_uncompressed()),
            chain: "TRX".to_string(),
            address_type: AddressType::Tron,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tron_generation() {
        let trx = Tron;
        let addr = trx.generate(AddressType::Tron);
        assert!(addr.address.starts_with("T"));
        assert_eq!(addr.chain, "TRX");
    }
}
