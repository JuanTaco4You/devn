//! Bitcoin Cash (BCH) CashAddr adapter
//!
//! BCH uses CashAddr format: bitcoincash:q...

use crate::traits::{Chain, ChainFamily, AddressType, GeneratedAddress};
use omnivanity_crypto::{
    Secp256k1Keypair,
    hash::hash160,
    encoding::wif_encode,
    hex,
};

/// Bitcoin Cash chain
pub struct BitcoinCash;

// CashAddr polymod checksum
fn cashaddr_polymod(values: &[u8]) -> u64 {
    let mut c: u64 = 1;
    for v in values {
        let c0 = (c >> 35) as u8;
        c = ((c & 0x07ffffffff) << 5) ^ (*v as u64);
        if c0 & 0x01 != 0 { c ^= 0x98f2bc8e61; }
        if c0 & 0x02 != 0 { c ^= 0x79b76d99e2; }
        if c0 & 0x04 != 0 { c ^= 0xf33e5fb3c4; }
        if c0 & 0x08 != 0 { c ^= 0xae2eabe2a8; }
        if c0 & 0x10 != 0 { c ^= 0x1e4f43e470; }
    }
    c ^ 1
}

fn cashaddr_encode(prefix: &str, payload: &[u8]) -> String {
    const CHARSET: &[u8] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";
    
    // Convert prefix to 5-bit values
    let mut values: Vec<u8> = prefix.bytes().map(|c| c & 0x1f).collect();
    values.push(0); // separator
    
    // Version byte (0 = P2PKH) + payload converted to 5-bit
    let mut payload_5bit = Vec::new();
    payload_5bit.push(0); // Version: P2PKH, 160-bit hash
    
    // Convert 8-bit payload to 5-bit
    let mut acc = 0u32;
    let mut bits = 0;
    for byte in payload {
        acc = (acc << 8) | (*byte as u32);
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            payload_5bit.push(((acc >> bits) & 0x1f) as u8);
        }
    }
    if bits > 0 {
        payload_5bit.push(((acc << (5 - bits)) & 0x1f) as u8);
    }
    
    values.extend(&payload_5bit);
    
    // Add checksum placeholder
    for _ in 0..8 {
        values.push(0);
    }
    
    let checksum = cashaddr_polymod(&values);
    let checksum_values: Vec<u8> = (0..8).map(|i| ((checksum >> (5 * (7 - i))) & 0x1f) as u8).collect();
    
    // Build result
    let mut result = String::from(prefix);
    result.push(':');
    for v in payload_5bit {
        result.push(CHARSET[v as usize] as char);
    }
    for v in checksum_values {
        result.push(CHARSET[v as usize] as char);
    }
    
    result
}

impl Chain for BitcoinCash {
    fn ticker(&self) -> &'static str {
        "BCH"
    }

    fn name(&self) -> &'static str {
        "Bitcoin Cash"
    }

    fn family(&self) -> ChainFamily {
        ChainFamily::UtxoSecp256k1
    }

    fn address_types(&self) -> Vec<AddressType> {
        vec![AddressType::CashAddr]
    }

    fn default_address_type(&self) -> AddressType {
        AddressType::CashAddr
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
        "qpzry9x8gf2tvdw0s3jn54khce6mua7l"
    }

    fn address_prefix(&self, _address_type: AddressType) -> &'static str {
        "bitcoincash:q"
    }
}

impl BitcoinCash {
    fn generate_from_keypair(&self, keypair: &Secp256k1Keypair, _address_type: AddressType) -> GeneratedAddress {
        let private_key = keypair.private_key_bytes();
        let pubkey = keypair.public_key_compressed();
        
        // BCH CashAddr: bitcoincash: + bech32-like encoding of hash160
        let h160 = hash160(&pubkey);
        let address = cashaddr_encode("bitcoincash", &h160);
        
        let wif = wif_encode(&private_key, true, true);
        
        GeneratedAddress {
            address,
            private_key_hex: hex::encode(private_key),
            private_key_native: wif,
            public_key_hex: hex::encode(pubkey),
            chain: "BCH".to_string(),
            address_type: AddressType::CashAddr,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bch_generation() {
        let bch = BitcoinCash;
        let addr = bch.generate(AddressType::CashAddr);
        assert!(addr.address.starts_with("bitcoincash:q"));
        assert_eq!(addr.chain, "BCH");
    }
}
