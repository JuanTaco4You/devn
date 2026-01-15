//! Zcash chain adapter (t-addresses only for now)

use crate::traits::{Chain, ChainFamily, AddressType, GeneratedAddress};
use omnivanity_crypto::{
    Secp256k1Keypair,
    hash::hash160,
    encoding::base58check_encode,
    hex, bs58,
};

/// Zcash chain (transparent addresses)
pub struct Zcash;

// Zcash version bytes (mainnet t-addresses)
// Zcash uses 2-byte version for addresses
const ZEC_T_ADDR_PREFIX: [u8; 2] = [0x1C, 0xB8]; // t1 prefix

impl Chain for Zcash {
    fn ticker(&self) -> &'static str {
        "ZEC"
    }

    fn name(&self) -> &'static str {
        "Zcash"
    }

    fn family(&self) -> ChainFamily {
        ChainFamily::UtxoSecp256k1
    }

    fn address_types(&self) -> Vec<AddressType> {
        vec![AddressType::P2pkh] // t-addr only for now
    }

    fn default_address_type(&self) -> AddressType {
        AddressType::P2pkh
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
        "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"
    }

    fn address_prefix(&self, _address_type: AddressType) -> &'static str {
        "t1"
    }
}

impl Zcash {
    fn generate_from_keypair(&self, keypair: &Secp256k1Keypair, _address_type: AddressType) -> GeneratedAddress {
        let private_key = keypair.private_key_bytes();
        let pubkey_compressed = keypair.public_key_compressed();
        
        // Zcash t-address: Base58Check with 2-byte prefix
        let h160 = hash160(&pubkey_compressed);
        let address = zec_t_addr_encode(&h160);
        
        // ZEC uses same WIF as BTC (0x80)
        let wif = omnivanity_crypto::encoding::wif_encode(&private_key, true, true);

        GeneratedAddress {
            address,
            private_key_hex: hex::encode(private_key),
            private_key_native: wif,
            public_key_hex: hex::encode(pubkey_compressed),
            chain: self.ticker().to_string(),
            address_type: AddressType::P2pkh,
        }
    }
}

fn zec_t_addr_encode(hash160: &[u8; 20]) -> String {
    use omnivanity_crypto::hash::double_sha256;
    
    // Zcash uses 2-byte version prefix
    let mut data = Vec::with_capacity(2 + 20 + 4);
    data.extend_from_slice(&ZEC_T_ADDR_PREFIX);
    data.extend_from_slice(hash160);
    
    let checksum = double_sha256(&data);
    data.extend_from_slice(&checksum[..4]);
    
    bs58::encode(data).into_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zec_t_addr() {
        let zec = Zcash;
        let addr = zec.generate(AddressType::P2pkh);
        assert!(addr.address.starts_with("t1"));
    }
}
