//! Bitcoin chain adapter

use crate::traits::{Chain, ChainFamily, AddressType, GeneratedAddress};
use omnivanity_crypto::{
    Secp256k1Keypair,
    hash::hash160,
    encoding::{base58check_encode, wif_encode, bech32_encode_v0},
    hex,
};

/// Bitcoin chain
pub struct Bitcoin;

impl Chain for Bitcoin {
    fn ticker(&self) -> &'static str {
        "BTC"
    }

    fn name(&self) -> &'static str {
        "Bitcoin"
    }

    fn family(&self) -> ChainFamily {
        ChainFamily::UtxoSecp256k1
    }

    fn address_types(&self) -> Vec<AddressType> {
        vec![AddressType::P2pkh, AddressType::P2wpkh, AddressType::P2tr]
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
            AddressType::P2wpkh | AddressType::P2tr => "023456789acdefghjklmnpqrstuvwxyz",
            _ => "",
        }
    }

    fn address_prefix(&self, address_type: AddressType) -> &'static str {
        match address_type {
            AddressType::P2pkh => "1",
            AddressType::P2sh => "3",
            AddressType::P2wpkh => "bc1q",
            AddressType::P2tr => "bc1p",
            _ => "",
        }
    }
}

impl Bitcoin {
    fn generate_from_keypair(&self, keypair: &Secp256k1Keypair, address_type: AddressType) -> GeneratedAddress {
        let private_key = keypair.private_key_bytes();
        let pubkey_compressed = keypair.public_key_compressed();
        
        let address = match address_type {
            AddressType::P2pkh => {
                // P2PKH: Base58Check(0x00 || HASH160(compressed_pubkey))
                let h160 = hash160(&pubkey_compressed);
                base58check_encode(0x00, &h160)
            }
            AddressType::P2wpkh => {
                // P2WPKH: bech32(bc, 0, HASH160(compressed_pubkey))
                let h160 = hash160(&pubkey_compressed);
                bech32_encode_v0("bc", &h160).unwrap_or_default()
            }
            AddressType::P2tr => {
                // Taproot: For now, simplified - real taproot needs tweaking
                // TODO: Implement proper taproot with key tweaking
                let h160 = hash160(&pubkey_compressed);
                bech32_encode_v0("bc", &h160).unwrap_or_default().replace("bc1q", "bc1p")
            }
            _ => String::new(),
        };

        let wif = wif_encode(&private_key, true, true);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_btc_p2pkh_generation() {
        let btc = Bitcoin;
        let addr = btc.generate(AddressType::P2pkh);
        
        assert!(addr.address.starts_with("1"));
        assert!(addr.private_key_native.starts_with("K") || addr.private_key_native.starts_with("L"));
    }

    #[test]
    fn test_btc_p2wpkh_generation() {
        let btc = Bitcoin;
        let addr = btc.generate(AddressType::P2wpkh);
        
        assert!(addr.address.starts_with("bc1q"));
    }

    #[test]
    fn test_known_vector() {
        let btc = Bitcoin;
        // Private key = 1, compressed
        let privkey = hex::decode("0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        let addr = btc.generate_from_bytes(&privkey, AddressType::P2pkh).unwrap();
        
        // Known P2PKH address for privkey=1 compressed
        assert_eq!(addr.address, "1BgGZ9tcN4rm9KBzDn7KprQz87SZ26SAMH");
        assert_eq!(addr.private_key_native, "KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU73sVHnoWn");
    }
}
