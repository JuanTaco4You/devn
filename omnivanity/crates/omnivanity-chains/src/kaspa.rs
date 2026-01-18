//! Kaspa chain adapter
//!
//! Kaspa uses Bech32 with kaspa: prefix

use crate::traits::{Chain, ChainFamily, AddressType, GeneratedAddress};
use omnivanity_crypto::{Secp256k1Keypair, hex};

/// Kaspa chain
pub struct Kaspa;

fn kaspa_bech32_encode(pubkey: &[u8]) -> Result<String, String> {
    use bech32::{Bech32, Hrp};
    // Kaspa schnorr pubkey (32 bytes x-only) with 0x00 prefix for ECDSA
    let hrp = Hrp::parse("kaspa").map_err(|e| e.to_string())?;
    
    // Prepend pubkey type byte (0x00 = ECDSA, 0x01 = Schnorr)
    let mut data = Vec::with_capacity(33);
    data.push(0x00); // ECDSA type
    data.extend_from_slice(&pubkey[1..33]); // Use x-coordinate from compressed pubkey
    
    bech32::encode::<Bech32>(hrp, &data).map_err(|e| e.to_string())
}

impl Chain for Kaspa {
    fn ticker(&self) -> &'static str {
        "KAS"
    }

    fn name(&self) -> &'static str {
        "Kaspa"
    }

    fn family(&self) -> ChainFamily {
        ChainFamily::UtxoSecp256k1
    }

    fn address_types(&self) -> Vec<AddressType> {
        vec![AddressType::Kaspa]
    }

    fn default_address_type(&self) -> AddressType {
        AddressType::Kaspa
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
        "kaspa:"
    }
}

impl Kaspa {
    fn generate_from_keypair(&self, keypair: &Secp256k1Keypair, _address_type: AddressType) -> GeneratedAddress {
        let private_key = keypair.private_key_bytes();
        let pubkey = keypair.public_key_compressed();
        
        let address = kaspa_bech32_encode(&pubkey).unwrap_or_default();
        
        GeneratedAddress {
            address,
            private_key_hex: hex::encode(private_key),
            private_key_native: hex::encode(private_key),
            public_key_hex: hex::encode(pubkey),
            chain: "KAS".to_string(),
            address_type: AddressType::Kaspa,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kaspa_generation() {
        let kas = Kaspa;
        let addr = kas.generate(AddressType::Kaspa);
        assert!(addr.address.starts_with("kaspa:"));
        assert_eq!(addr.chain, "KAS");
    }
}
