//! Cardano chain adapter  
//!
//! Cardano Shelley-era addresses: Bech32 addr1...

use crate::traits::{Chain, ChainFamily, AddressType, GeneratedAddress};
use omnivanity_crypto::{Ed25519Keypair, hash::blake2b_224, hex};

/// Cardano chain
pub struct Cardano;

fn cardano_bech32_encode(hrp: &str, data: &[u8]) -> Result<String, String> {
    use bech32::{Bech32, Hrp};
    let hrp = Hrp::parse(hrp).map_err(|e| e.to_string())?;
    bech32::encode::<Bech32>(hrp, data).map_err(|e| e.to_string())
}

impl Chain for Cardano {
    fn ticker(&self) -> &'static str {
        "ADA"
    }

    fn name(&self) -> &'static str {
        "Cardano"
    }

    fn family(&self) -> ChainFamily {
        ChainFamily::Ed25519
    }

    fn address_types(&self) -> Vec<AddressType> {
        vec![AddressType::Cardano]
    }

    fn default_address_type(&self) -> AddressType {
        AddressType::Cardano
    }

    fn generate(&self, address_type: AddressType) -> GeneratedAddress {
        let keypair = Ed25519Keypair::generate();
        self.generate_from_keypair(&keypair, address_type)
    }

    fn generate_from_bytes(&self, private_key: &[u8], address_type: AddressType) -> Option<GeneratedAddress> {
        if private_key.len() != 32 {
            return None;
        }
        let mut pk = [0u8; 32];
        pk.copy_from_slice(private_key);
        let keypair = Ed25519Keypair::from_bytes(&pk).ok()?;
        Some(self.generate_from_keypair(&keypair, address_type))
    }

    fn valid_address_chars(&self, _address_type: AddressType) -> &'static str {
        "023456789acdefghjklmnpqrstuvwxyz"
    }

    fn address_prefix(&self, _address_type: AddressType) -> &'static str {
        "addr1"
    }
}

impl Cardano {
    fn generate_from_keypair(&self, keypair: &Ed25519Keypair, _address_type: AddressType) -> GeneratedAddress {
        let private_key = keypair.private_key_bytes();
        let public_key = keypair.public_key_bytes();
        
        // Cardano Shelley base address (simplified):
        // Header byte (0x01 = base address, mainnet) + payment key hash (28 bytes) + stake key hash (28 bytes)
        // For simplicity, we'll generate an enterprise address (no staking, 0x61 header)
        let payment_hash = blake2b_224(&public_key);
        
        let mut addr_bytes = Vec::with_capacity(29);
        addr_bytes.push(0x61); // Enterprise address, mainnet
        addr_bytes.extend_from_slice(&payment_hash);
        
        let address = cardano_bech32_encode("addr", &addr_bytes).unwrap_or_default();
        
        GeneratedAddress {
            address,
            private_key_hex: hex::encode(private_key),
            private_key_native: hex::encode(private_key),
            public_key_hex: hex::encode(public_key),
            chain: "ADA".to_string(),
            address_type: AddressType::Cardano,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cardano_generation() {
        let ada = Cardano;
        let addr = ada.generate(AddressType::Cardano);
        assert!(addr.address.starts_with("addr1"));
        assert_eq!(addr.chain, "ADA");
    }
}
