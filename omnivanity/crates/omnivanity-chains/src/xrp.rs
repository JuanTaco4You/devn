//! XRP Ledger chain adapter
//!
//! XRP classic address: secp256k1, RIPEMD160(SHA256(pubkey)), XRPL Base58 encoding

use crate::traits::{Chain, ChainFamily, AddressType, GeneratedAddress};
use omnivanity_crypto::{
    Secp256k1Keypair,
    hash::hash160,
    hex,
};

/// XRP Ledger chain
pub struct Xrp;

// XRPL uses a different Base58 alphabet
const XRPL_ALPHABET: &[u8] = b"rpshnaf39wBUDNEGHJKLM4PQRST7VWXYZ2bcdeCg65jkm8oFqi1tuvAxyz";

fn xrpl_base58_encode(data: &[u8]) -> String {
    // Convert to XRPL alphabet
    let standard = bs58::encode(data).into_string();
    let standard_alphabet = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    
    standard.chars().map(|c| {
        if let Some(idx) = standard_alphabet.iter().position(|&x| x == c as u8) {
            XRPL_ALPHABET[idx] as char
        } else {
            c
        }
    }).collect()
}

fn xrpl_base58check_encode(version: u8, payload: &[u8]) -> String {
    use omnivanity_crypto::hash::double_sha256;
    
    let mut data = Vec::with_capacity(1 + payload.len() + 4);
    data.push(version);
    data.extend_from_slice(payload);
    
    let checksum = double_sha256(&data);
    data.extend_from_slice(&checksum[..4]);
    
    xrpl_base58_encode(&data)
}

impl Chain for Xrp {
    fn ticker(&self) -> &'static str {
        "XRP"
    }

    fn name(&self) -> &'static str {
        "XRP Ledger"
    }

    fn family(&self) -> ChainFamily {
        ChainFamily::UtxoSecp256k1
    }

    fn address_types(&self) -> Vec<AddressType> {
        vec![AddressType::Xrpl]
    }

    fn default_address_type(&self) -> AddressType {
        AddressType::Xrpl
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
        "rpshnaf39wBUDNEGHJKLM4PQRST7VWXYZ2bcdeCg65jkm8oFqi1tuvAxyz"
    }

    fn address_prefix(&self, _address_type: AddressType) -> &'static str {
        "r"
    }
}

impl Xrp {
    fn generate_from_keypair(&self, keypair: &Secp256k1Keypair, _address_type: AddressType) -> GeneratedAddress {
        let private_key = keypair.private_key_bytes();
        let pubkey_compressed = keypair.public_key_compressed();
        
        // XRP address: version byte 0x00, RIPEMD160(SHA256(pubkey))
        let h160 = hash160(&pubkey_compressed);
        let address = xrpl_base58check_encode(0x00, &h160);
        
        GeneratedAddress {
            address,
            private_key_hex: hex::encode(private_key),
            private_key_native: hex::encode(private_key),
            public_key_hex: hex::encode(pubkey_compressed),
            chain: "XRP".to_string(),
            address_type: AddressType::Xrpl,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xrp_generation() {
        let xrp = Xrp;
        let addr = xrp.generate(AddressType::Xrpl);
        assert!(addr.address.starts_with("r"));
        assert_eq!(addr.chain, "XRP");
    }
}
