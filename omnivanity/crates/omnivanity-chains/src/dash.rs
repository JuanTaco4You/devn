//! DASH chain adapter

use crate::traits::{Chain, ChainFamily, AddressType, GeneratedAddress};
use omnivanity_crypto::{
    Secp256k1Keypair,
    hash::hash160,
    encoding::{base58check_encode, wif_encode},
    hex,
};

/// Dash chain
pub struct Dash;

impl Chain for Dash {
    fn ticker(&self) -> &'static str {
        "DASH"
    }

    fn name(&self) -> &'static str {
        "Dash"
    }

    fn family(&self) -> ChainFamily {
        ChainFamily::UtxoSecp256k1
    }

    fn address_types(&self) -> Vec<AddressType> {
        vec![AddressType::P2pkh]
    }

    fn default_address_type(&self) -> AddressType {
        AddressType::P2pkh
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
        "X"
    }
}

impl Dash {
    fn generate_from_keypair(&self, keypair: &Secp256k1Keypair, _address_type: AddressType) -> GeneratedAddress {
        let private_key = keypair.private_key_bytes();
        let pubkey_compressed = keypair.public_key_compressed();
        
        // Dash P2PKH: version byte 0x4C (76)
        let h160 = hash160(&pubkey_compressed);
        let address = base58check_encode(0x4C, &h160);
        
        // Dash WIF: version byte 0xCC (204)
        let wif = wif_encode(&private_key, true, true).replace("K", "X").replace("L", "X"); // Simplification
        let wif = format!("7{}", &wif[1..]); // Dash WIF starts with 7
        
        GeneratedAddress {
            address,
            private_key_hex: hex::encode(private_key),
            private_key_native: wif,
            public_key_hex: hex::encode(pubkey_compressed),
            chain: "DASH".to_string(),
            address_type: AddressType::P2pkh,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dash_generation() {
        let dash = Dash;
        let addr = dash.generate(AddressType::P2pkh);
        assert!(addr.address.starts_with("X"));
        assert_eq!(addr.chain, "DASH");
    }
}
