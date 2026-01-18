//! Ravencoin chain adapter

use crate::traits::{Chain, ChainFamily, AddressType, GeneratedAddress};
use omnivanity_crypto::{
    Secp256k1Keypair,
    hash::hash160,
    encoding::{base58check_encode, wif_encode},
    hex,
};

/// Ravencoin chain
pub struct Ravencoin;

impl Chain for Ravencoin {
    fn ticker(&self) -> &'static str {
        "RVN"
    }

    fn name(&self) -> &'static str {
        "Ravencoin"
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
        "R"
    }
}

impl Ravencoin {
    fn generate_from_keypair(&self, keypair: &Secp256k1Keypair, _address_type: AddressType) -> GeneratedAddress {
        let private_key = keypair.private_key_bytes();
        let pubkey_compressed = keypair.public_key_compressed();
        
        // Ravencoin P2PKH: version byte 0x3C (60)
        let h160 = hash160(&pubkey_compressed);
        let address = base58check_encode(0x3C, &h160);
        
        let wif = wif_encode(&private_key, true, true);
        
        GeneratedAddress {
            address,
            private_key_hex: hex::encode(private_key),
            private_key_native: wif,
            public_key_hex: hex::encode(pubkey_compressed),
            chain: "RVN".to_string(),
            address_type: AddressType::P2pkh,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rvn_generation() {
        let rvn = Ravencoin;
        let addr = rvn.generate(AddressType::P2pkh);
        assert!(addr.address.starts_with("R"));
        assert_eq!(addr.chain, "RVN");
    }
}
