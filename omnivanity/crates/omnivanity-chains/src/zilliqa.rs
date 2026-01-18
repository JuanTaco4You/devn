//! Zilliqa chain adapter
//!
//! Zilliqa uses Bech32 (zil1...) for display addresses

use crate::traits::{Chain, ChainFamily, AddressType, GeneratedAddress};
use omnivanity_crypto::{Secp256k1Keypair, hash::sha256, hex};

/// Zilliqa chain
pub struct Zilliqa;

fn zil_bech32_encode(data: &[u8]) -> Result<String, String> {
    use bech32::{Bech32, Hrp};
    let hrp = Hrp::parse("zil").map_err(|e| e.to_string())?;
    bech32::encode::<Bech32>(hrp, data).map_err(|e| e.to_string())
}

impl Chain for Zilliqa {
    fn ticker(&self) -> &'static str {
        "ZIL"
    }

    fn name(&self) -> &'static str {
        "Zilliqa"
    }

    fn family(&self) -> ChainFamily {
        ChainFamily::UtxoSecp256k1
    }

    fn address_types(&self) -> Vec<AddressType> {
        vec![AddressType::Zilliqa]
    }

    fn default_address_type(&self) -> AddressType {
        AddressType::Zilliqa
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
        "023456789acdefghjklmnpqrstuvwxyz"
    }

    fn address_prefix(&self, _address_type: AddressType) -> &'static str {
        "zil1"
    }
}

impl Zilliqa {
    fn generate_from_keypair(&self, keypair: &Secp256k1Keypair, _address_type: AddressType) -> GeneratedAddress {
        let private_key = keypair.private_key_bytes();
        let pubkey = keypair.public_key_compressed();
        
        // Zilliqa: SHA256(compressed_pubkey), take last 20 bytes
        let hash = sha256(&pubkey);
        let address_bytes = &hash[12..32];
        
        // Encode as bech32
        let address = zil_bech32_encode(address_bytes).unwrap_or_default();
        
        GeneratedAddress {
            address,
            private_key_hex: hex::encode(private_key),
            private_key_native: hex::encode(private_key),
            public_key_hex: hex::encode(pubkey),
            chain: "ZIL".to_string(),
            address_type: AddressType::Zilliqa,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zil_generation() {
        let zil = Zilliqa;
        let addr = zil.generate(AddressType::Zilliqa);
        assert!(addr.address.starts_with("zil1"));
        assert_eq!(addr.chain, "ZIL");
    }
}
