//! Litecoin chain adapter

use crate::traits::{Chain, ChainFamily, AddressType, GeneratedAddress};
use omnivanity_crypto::{
    Secp256k1Keypair,
    hash::hash160,
    encoding::{base58check_encode, bech32_encode_v0},
    hex,
};

/// Litecoin chain
pub struct Litecoin;

// Litecoin version bytes
const LTC_P2PKH_VERSION: u8 = 0x30; // L prefix
const LTC_P2SH_VERSION: u8 = 0x32;  // M prefix (or 0x05 for 3 prefix)
const LTC_WIF_VERSION: u8 = 0xB0;

impl Chain for Litecoin {
    fn ticker(&self) -> &'static str {
        "LTC"
    }

    fn name(&self) -> &'static str {
        "Litecoin"
    }

    fn family(&self) -> ChainFamily {
        ChainFamily::UtxoSecp256k1
    }

    fn address_types(&self) -> Vec<AddressType> {
        vec![AddressType::P2pkh, AddressType::P2wpkh]
    }

    fn default_address_type(&self) -> AddressType {
        AddressType::P2wpkh
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

    fn valid_address_chars(&self, address_type: AddressType) -> &'static str {
        match address_type {
            AddressType::P2pkh | AddressType::P2sh => "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz",
            AddressType::P2wpkh => "023456789acdefghjklmnpqrstuvwxyz",
            _ => "",
        }
    }

    fn address_prefix(&self, address_type: AddressType) -> &'static str {
        match address_type {
            AddressType::P2pkh => "L",
            AddressType::P2sh => "M",
            AddressType::P2wpkh => "ltc1q",
            _ => "",
        }
    }
}

impl Litecoin {
    fn generate_from_keypair(&self, keypair: &Secp256k1Keypair, address_type: AddressType) -> GeneratedAddress {
        let private_key = keypair.private_key_bytes();
        let pubkey_compressed = keypair.public_key_compressed();
        
        let address = match address_type {
            AddressType::P2pkh => {
                let h160 = hash160(&pubkey_compressed);
                base58check_encode(LTC_P2PKH_VERSION, &h160)
            }
            AddressType::P2wpkh => {
                let h160 = hash160(&pubkey_compressed);
                bech32_encode_v0("ltc", &h160).unwrap_or_default()
            }
            _ => String::new(),
        };

        // LTC WIF
        let wif = ltc_wif_encode(&private_key, true);

        GeneratedAddress {
            address,
            private_key_hex: hex::encode(private_key),
            private_key_native: wif,
            public_key_hex: hex::encode(pubkey_compressed),
            chain: self.ticker().to_string(),
            address_type,
        }
    }
}

fn ltc_wif_encode(private_key: &[u8; 32], compressed: bool) -> String {
    use omnivanity_crypto::encoding::base58check_encode;
    
    if compressed {
        let mut payload = Vec::with_capacity(33);
        payload.extend_from_slice(private_key);
        payload.push(0x01);
        base58check_encode(LTC_WIF_VERSION, &payload)
    } else {
        base58check_encode(LTC_WIF_VERSION, private_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ltc_p2pkh() {
        let ltc = Litecoin;
        let addr = ltc.generate(AddressType::P2pkh);
        assert!(addr.address.starts_with("L"));
    }

    #[test]
    fn test_ltc_p2wpkh() {
        let ltc = Litecoin;
        let addr = ltc.generate(AddressType::P2wpkh);
        assert!(addr.address.starts_with("ltc1q"));
    }
}
