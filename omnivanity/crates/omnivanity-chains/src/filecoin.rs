//! Filecoin chain adapter
//!
//! Filecoin f1 addresses: Blake2b-160(pubkey) + Base32 encoding with checksum

use crate::traits::{Chain, ChainFamily, AddressType, GeneratedAddress};
use omnivanity_crypto::{Secp256k1Keypair, hash::blake2b_160, hex};
use blake2::{Blake2b, Digest};
use blake2::digest::consts::U4;

/// Filecoin chain
pub struct Filecoin;

// Filecoin uses lowercase base32
const FIL_BASE32_ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz234567";

fn fil_base32_encode(data: &[u8]) -> String {
    let mut result = String::new();
    let mut bits = 0u32;
    let mut value = 0u32;
    
    for &byte in data {
        value = (value << 8) | (byte as u32);
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            result.push(FIL_BASE32_ALPHABET[((value >> bits) & 0x1F) as usize] as char);
        }
    }
    
    if bits > 0 {
        result.push(FIL_BASE32_ALPHABET[((value << (5 - bits)) & 0x1F) as usize] as char);
    }
    
    result
}

fn fil_checksum(protocol: u8, payload: &[u8]) -> [u8; 4] {
    type Blake2b32 = Blake2b<U4>;
    let mut hasher = Blake2b32::new();
    hasher.update(&[protocol]);
    hasher.update(payload);
    let result = hasher.finalize();
    let mut checksum = [0u8; 4];
    checksum.copy_from_slice(&result);
    checksum
}

impl Chain for Filecoin {
    fn ticker(&self) -> &'static str {
        "FIL"
    }

    fn name(&self) -> &'static str {
        "Filecoin"
    }

    fn family(&self) -> ChainFamily {
        ChainFamily::UtxoSecp256k1
    }

    fn address_types(&self) -> Vec<AddressType> {
        vec![AddressType::Filecoin]
    }

    fn default_address_type(&self) -> AddressType {
        AddressType::Filecoin
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
        "abcdefghijklmnopqrstuvwxyz234567"
    }

    fn address_prefix(&self, _address_type: AddressType) -> &'static str {
        "f1"
    }
}

impl Filecoin {
    fn generate_from_keypair(&self, keypair: &Secp256k1Keypair, _address_type: AddressType) -> GeneratedAddress {
        let private_key = keypair.private_key_bytes();
        let pubkey = keypair.public_key_uncompressed();
        
        // Protocol 1 (secp256k1): payload = Blake2b-160(uncompressed_pubkey)
        let payload = blake2b_160(&pubkey);
        
        // Checksum
        let checksum = fil_checksum(1, &payload);
        
        // Address = f1 + base32(payload + checksum)
        let mut data = Vec::with_capacity(24);
        data.extend_from_slice(&payload);
        data.extend_from_slice(&checksum);
        
        let address = format!("f1{}", fil_base32_encode(&data));
        
        GeneratedAddress {
            address,
            private_key_hex: hex::encode(private_key),
            private_key_native: hex::encode(private_key),
            public_key_hex: hex::encode(pubkey),
            chain: "FIL".to_string(),
            address_type: AddressType::Filecoin,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fil_generation() {
        let fil = Filecoin;
        let addr = fil.generate(AddressType::Filecoin);
        assert!(addr.address.starts_with("f1"));
        assert_eq!(addr.chain, "FIL");
    }
}
