//! Cosmos-SDK Bech32 chain family
//!
//! Covers: ATOM, OSMO, INJ, SEI, TIA, JUNO, KAVA, SCRT, RUNE, CRO, etc.
//! All use: secp256k1 + RIPEMD160(SHA256(pubkey)) + Bech32 with chain-specific HRP

use crate::traits::{Chain, ChainFamily, AddressType, GeneratedAddress};
use omnivanity_crypto::{
    Secp256k1Keypair,
    hash::hash160,
    encoding::bech32_encode_v0,
    hex,
};

/// Cosmos-style chain with configurable HRP
#[derive(Debug, Clone, Copy)]
pub struct CosmosChain {
    ticker: &'static str,
    name: &'static str,
    hrp: &'static str,
}

impl CosmosChain {
    pub const fn new(ticker: &'static str, name: &'static str, hrp: &'static str) -> Self {
        Self { ticker, name, hrp }
    }

    fn generate_from_keypair(&self, keypair: &Secp256k1Keypair, _address_type: AddressType) -> GeneratedAddress {
        let private_key = keypair.private_key_bytes();
        let pubkey_compressed = keypair.public_key_compressed();
        
        // Cosmos address = bech32(hrp, RIPEMD160(SHA256(compressed_pubkey)))
        let h160 = hash160(&pubkey_compressed);
        let address = bech32_encode_v0(self.hrp, &h160).unwrap_or_default();
        
        GeneratedAddress {
            address,
            private_key_hex: hex::encode(private_key),
            private_key_native: hex::encode(private_key), // Cosmos typically uses hex
            public_key_hex: hex::encode(pubkey_compressed),
            chain: self.ticker.to_string(),
            address_type: AddressType::Cosmos,
        }
    }
}

// Pre-defined Cosmos chains
pub const ATOM: CosmosChain = CosmosChain::new("ATOM", "Cosmos Hub", "cosmos");
pub const OSMO: CosmosChain = CosmosChain::new("OSMO", "Osmosis", "osmo");
pub const INJ: CosmosChain = CosmosChain::new("INJ", "Injective", "inj");
pub const SEI: CosmosChain = CosmosChain::new("SEI", "Sei", "sei");
pub const TIA: CosmosChain = CosmosChain::new("TIA", "Celestia", "celestia");
pub const JUNO: CosmosChain = CosmosChain::new("JUNO", "Juno", "juno");
pub const KAVA: CosmosChain = CosmosChain::new("KAVA", "Kava", "kava");
pub const SCRT: CosmosChain = CosmosChain::new("SCRT", "Secret Network", "secret");
pub const RUNE: CosmosChain = CosmosChain::new("RUNE", "THORChain", "thor");
pub const CRO: CosmosChain = CosmosChain::new("CRO", "Crypto.org Chain", "cro");

impl Chain for CosmosChain {
    fn ticker(&self) -> &'static str {
        self.ticker
    }

    fn name(&self) -> &'static str {
        self.name
    }

    fn family(&self) -> ChainFamily {
        ChainFamily::UtxoSecp256k1 // Uses secp256k1 like Bitcoin
    }

    fn address_types(&self) -> Vec<AddressType> {
        vec![AddressType::Cosmos]
    }

    fn default_address_type(&self) -> AddressType {
        AddressType::Cosmos
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
        // Bech32 uses lowercase alphanumeric except 1, b, i, o
        "023456789acdefghjklmnpqrstuvwxyz"
    }

    fn address_prefix(&self, _address_type: AddressType) -> &'static str {
        self.hrp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atom_generation() {
        let addr = ATOM.generate(AddressType::Cosmos);
        assert!(addr.address.starts_with("cosmos1"));
        assert_eq!(addr.chain, "ATOM");
    }

    #[test]
    fn test_osmo_generation() {
        let addr = OSMO.generate(AddressType::Cosmos);
        assert!(addr.address.starts_with("osmo1"));
        assert_eq!(addr.chain, "OSMO");
    }

    #[test]
    fn test_inj_generation() {
        let addr = INJ.generate(AddressType::Cosmos);
        assert!(addr.address.starts_with("inj1"));
        assert_eq!(addr.chain, "INJ");
    }

    #[test]
    fn test_sei_generation() {
        let addr = SEI.generate(AddressType::Cosmos);
        assert!(addr.address.starts_with("sei1"));
        assert_eq!(addr.chain, "SEI");
    }
}
