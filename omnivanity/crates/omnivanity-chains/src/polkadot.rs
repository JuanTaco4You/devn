//! Polkadot/Substrate SS58 chain family
//!
//! SS58: network prefix + pubkey + checksum, Base58 encoded

use crate::traits::{Chain, ChainFamily, AddressType, GeneratedAddress};
use omnivanity_crypto::{Ed25519Keypair, hex};
use blake2::{Blake2b512, Digest};

/// SS58 chain with configurable network prefix
#[derive(Debug, Clone, Copy)]
pub struct Ss58Chain {
    ticker: &'static str,
    name: &'static str,
    prefix: u16,
}

impl Ss58Chain {
    pub const fn new(ticker: &'static str, name: &'static str, prefix: u16) -> Self {
        Self { ticker, name, prefix }
    }
}

// SS58 checksum prefix
const SS58_PREFIX: &[u8] = b"SS58PRE";

fn ss58_encode(prefix: u16, pubkey: &[u8; 32]) -> String {
    let mut data = Vec::with_capacity(35);
    
    // Add network prefix (1 or 2 bytes)
    if prefix < 64 {
        data.push(prefix as u8);
    } else if prefix < 16384 {
        data.push(((prefix & 0x00FC) >> 2) as u8 | 0x40);
        data.push((((prefix >> 8) & 0xFF) | ((prefix & 0x03) << 6)) as u8);
    } else {
        panic!("Invalid SS58 prefix");
    }
    
    // Add public key
    data.extend_from_slice(pubkey);
    
    // Calculate checksum
    let mut hasher = Blake2b512::new();
    hasher.update(SS58_PREFIX);
    hasher.update(&data);
    let hash = hasher.finalize();
    
    // Add first 2 bytes of checksum
    data.push(hash[0]);
    data.push(hash[1]);
    
    bs58::encode(data).into_string()
}

// Pre-defined Substrate chains
pub const DOT: Ss58Chain = Ss58Chain::new("DOT", "Polkadot", 0);
pub const KSM: Ss58Chain = Ss58Chain::new("KSM", "Kusama", 2);
pub const ACA: Ss58Chain = Ss58Chain::new("ACA", "Acala", 10);
pub const CFG: Ss58Chain = Ss58Chain::new("CFG", "Centrifuge", 36);
pub const HDX: Ss58Chain = Ss58Chain::new("HDX", "HydraDX", 63);

impl Chain for Ss58Chain {
    fn ticker(&self) -> &'static str {
        self.ticker
    }

    fn name(&self) -> &'static str {
        self.name
    }

    fn family(&self) -> ChainFamily {
        ChainFamily::Ed25519
    }

    fn address_types(&self) -> Vec<AddressType> {
        vec![AddressType::Ss58]
    }

    fn default_address_type(&self) -> AddressType {
        AddressType::Ss58
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
        "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"
    }

    fn address_prefix(&self, _address_type: AddressType) -> &'static str {
        match self.prefix {
            0 => "1",  // Polkadot starts with 1
            2 => "C",  // Kusama starts with C/D/E/F/G/H/J
            _ => "",
        }
    }
}

impl Ss58Chain {
    fn generate_from_keypair(&self, keypair: &Ed25519Keypair, _address_type: AddressType) -> GeneratedAddress {
        let private_key = keypair.private_key_bytes();
        let public_key = keypair.public_key_bytes();
        
        let address = ss58_encode(self.prefix, &public_key);
        
        GeneratedAddress {
            address,
            private_key_hex: hex::encode(private_key),
            private_key_native: hex::encode(private_key),
            public_key_hex: hex::encode(public_key),
            chain: self.ticker.to_string(),
            address_type: AddressType::Ss58,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dot_generation() {
        let addr = DOT.generate(AddressType::Ss58);
        assert!(addr.address.starts_with("1"));
        assert_eq!(addr.chain, "DOT");
    }

    #[test]
    fn test_ksm_generation() {
        let addr = KSM.generate(AddressType::Ss58);
        assert_eq!(addr.chain, "KSM");
    }
}
