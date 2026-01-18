//! XDC Network chain adapter
//!
//! XDC uses EVM-style addresses but with 'xdc' prefix instead of '0x'

use crate::traits::{Chain, ChainFamily, AddressType, GeneratedAddress};
use omnivanity_crypto::{Secp256k1Keypair, hash::keccak256, hex};

/// XDC Network chain
pub struct Xdc;

impl Chain for Xdc {
    fn ticker(&self) -> &'static str {
        "XDC"
    }

    fn name(&self) -> &'static str {
        "XDC Network"
    }

    fn family(&self) -> ChainFamily {
        ChainFamily::Evm
    }

    fn address_types(&self) -> Vec<AddressType> {
        vec![AddressType::Xdc]
    }

    fn default_address_type(&self) -> AddressType {
        AddressType::Xdc
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
        "0123456789abcdef"
    }

    fn address_prefix(&self, _address_type: AddressType) -> &'static str {
        "xdc"
    }
}

impl Xdc {
    fn generate_from_keypair(&self, keypair: &Secp256k1Keypair, _address_type: AddressType) -> GeneratedAddress {
        let private_key = keypair.private_key_bytes();
        
        // XDC address = xdc + last 20 bytes of keccak256(uncompressed_pubkey[1..65])
        let pubkey_xy = keypair.public_key_xy();
        let hash = keccak256(&pubkey_xy);
        
        let mut address_bytes = [0u8; 20];
        address_bytes.copy_from_slice(&hash[12..32]);
        
        let address = format!("xdc{}", hex::encode(address_bytes));
        
        GeneratedAddress {
            address,
            private_key_hex: format!("0x{}", hex::encode(private_key)),
            private_key_native: format!("0x{}", hex::encode(private_key)),
            public_key_hex: format!("0x{}", hex::encode(keypair.public_key_uncompressed())),
            chain: "XDC".to_string(),
            address_type: AddressType::Xdc,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xdc_generation() {
        let xdc = Xdc;
        let addr = xdc.generate(AddressType::Xdc);
        assert!(addr.address.starts_with("xdc"));
        assert_eq!(addr.address.len(), 43); // xdc + 40 hex chars
        assert_eq!(addr.chain, "XDC");
    }
}
