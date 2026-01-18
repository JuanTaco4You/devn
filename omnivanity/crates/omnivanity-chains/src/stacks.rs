//! Stacks chain adapter
//!
//! Stacks uses c32check encoding (Crockford base32 variant with checksum)

use crate::traits::{Chain, ChainFamily, AddressType, GeneratedAddress};
use omnivanity_crypto::{Secp256k1Keypair, hash::{sha256, hash160}, hex};

/// Stacks chain
pub struct Stacks;

// c32 alphabet (Crockford's base32, lowercase variant)
const C32_ALPHABET: &[u8] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

fn c32_encode(data: &[u8]) -> String {
    let mut result = String::new();
    let mut bits = 0u32;
    let mut value = 0u64;
    
    for &byte in data {
        value = (value << 8) | (byte as u64);
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            result.push(C32_ALPHABET[((value >> bits) & 0x1F) as usize] as char);
        }
    }
    
    if bits > 0 {
        result.push(C32_ALPHABET[((value << (5 - bits)) & 0x1F) as usize] as char);
    }
    
    result
}

fn c32check_encode(version: u8, data: &[u8]) -> String {
    // Checksum: first 4 bytes of sha256(sha256(version || data))
    let mut payload = Vec::with_capacity(1 + data.len());
    payload.push(version);
    payload.extend_from_slice(data);
    
    let checksum = sha256(&sha256(&payload));
    payload.extend_from_slice(&checksum[..4]);
    
    // Encode with c32
    let encoded = c32_encode(&payload);
    
    // Add prefix based on version
    match version {
        22 => format!("SP{}", encoded), // Mainnet single-sig
        20 => format!("SM{}", encoded), // Mainnet multi-sig
        26 => format!("ST{}", encoded), // Testnet single-sig
        21 => format!("SN{}", encoded), // Testnet multi-sig
        _ => format!("S{}", encoded),
    }
}

impl Chain for Stacks {
    fn ticker(&self) -> &'static str {
        "STX"
    }

    fn name(&self) -> &'static str {
        "Stacks"
    }

    fn family(&self) -> ChainFamily {
        ChainFamily::UtxoSecp256k1
    }

    fn address_types(&self) -> Vec<AddressType> {
        vec![AddressType::Stacks]
    }

    fn default_address_type(&self) -> AddressType {
        AddressType::Stacks
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
        "0123456789ABCDEFGHJKMNPQRSTVWXYZ"
    }

    fn address_prefix(&self, _address_type: AddressType) -> &'static str {
        "SP"
    }
}

impl Stacks {
    fn generate_from_keypair(&self, keypair: &Secp256k1Keypair, _address_type: AddressType) -> GeneratedAddress {
        let private_key = keypair.private_key_bytes();
        let pubkey_compressed = keypair.public_key_compressed();
        
        // Stacks address: c32check(version, hash160(pubkey))
        let h160 = hash160(&pubkey_compressed);
        let address = c32check_encode(22, &h160); // 22 = mainnet single-sig
        
        GeneratedAddress {
            address,
            private_key_hex: hex::encode(private_key),
            private_key_native: hex::encode(private_key),
            public_key_hex: hex::encode(pubkey_compressed),
            chain: "STX".to_string(),
            address_type: AddressType::Stacks,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stacks_generation() {
        let stx = Stacks;
        let addr = stx.generate(AddressType::Stacks);
        assert!(addr.address.starts_with("SP"));
        assert_eq!(addr.chain, "STX");
    }
}
