//! Dogecoin chain adapter

use crate::traits::{Chain, ChainFamily, AddressType, GeneratedAddress};
use omnivanity_crypto::{
    Secp256k1Keypair,
    hash::hash160,
    encoding::base58check_encode,
    hex,
};

/// Dogecoin chain
pub struct Dogecoin;

// Dogecoin version bytes
const DOGE_P2PKH_VERSION: u8 = 0x1E; // D prefix
const DOGE_P2SH_VERSION: u8 = 0x16;  // 9 or A prefix
const DOGE_WIF_VERSION: u8 = 0x9E;

impl Chain for Dogecoin {
    fn ticker(&self) -> &'static str {
        "DOGE"
    }

    fn name(&self) -> &'static str {
        "Dogecoin"
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
        "D"
    }
}

impl Dogecoin {
    fn generate_from_keypair(&self, keypair: &Secp256k1Keypair, _address_type: AddressType) -> GeneratedAddress {
        let private_key = keypair.private_key_bytes();
        let pubkey_compressed = keypair.public_key_compressed();
        
        let h160 = hash160(&pubkey_compressed);
        let address = base58check_encode(DOGE_P2PKH_VERSION, &h160);
        
        // DOGE WIF
        let wif = doge_wif_encode(&private_key, true);

        GeneratedAddress {
            address,
            private_key_hex: hex::encode(private_key),
            private_key_native: wif,
            public_key_hex: hex::encode(pubkey_compressed),
            chain: self.ticker().to_string(),
            address_type: AddressType::P2pkh,
        }
    }
}

fn doge_wif_encode(private_key: &[u8; 32], compressed: bool) -> String {
    if compressed {
        let mut payload = Vec::with_capacity(33);
        payload.extend_from_slice(private_key);
        payload.push(0x01);
        base58check_encode(DOGE_WIF_VERSION, &payload)
    } else {
        base58check_encode(DOGE_WIF_VERSION, private_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doge_p2pkh() {
        let doge = Dogecoin;
        let addr = doge.generate(AddressType::P2pkh);
        assert!(addr.address.starts_with("D"));
    }
}
