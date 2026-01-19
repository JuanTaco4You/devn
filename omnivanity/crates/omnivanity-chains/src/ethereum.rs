//! EVM chain family
//!
//! Covers: ETH, BNB, MATIC, ARB, OP, AVAX, FTM, GNO, CELO, etc.
//! All use: secp256k1 + Keccak-256(pubkey[1..65]) last 20 bytes + EIP-55 checksum

use crate::traits::{Chain, ChainFamily, AddressType, GeneratedAddress};
use omnivanity_crypto::{Secp256k1Keypair, hash::keccak256, encoding::eip55_checksum, hex};

/// EVM-compatible chain with configurable ticker/name
#[derive(Debug, Clone, Copy)]
pub struct EvmChain {
    ticker: &'static str,
    name: &'static str,
}

impl EvmChain {
    pub const fn new(ticker: &'static str, name: &'static str) -> Self {
        Self { ticker, name }
    }

    fn generate_from_keypair(&self, keypair: &Secp256k1Keypair, _address_type: AddressType) -> GeneratedAddress {
        // ETH address = last 20 bytes of keccak256(uncompressed_pubkey[1..65])
        let pubkey_xy = keypair.public_key_xy();
        let hash = keccak256(&pubkey_xy);
        
        let mut address_bytes = [0u8; 20];
        address_bytes.copy_from_slice(&hash[12..32]);
        
        let address = eip55_checksum(&address_bytes);
        let private_key = keypair.private_key_bytes();
        
        GeneratedAddress {
            address,
            private_key_hex: format!("0x{}", hex::encode(private_key)),
            private_key_native: format!("0x{}", hex::encode(private_key)),
            public_key_hex: format!("0x{}", hex::encode(keypair.public_key_uncompressed())),
            chain: self.ticker.to_string(),
            address_type: AddressType::Evm,
        }
    }
}

// Pre-defined EVM chains and tokens
// Native EVM chains
pub const ETH: EvmChain = EvmChain::new("ETH", "Ethereum");
pub const BNB: EvmChain = EvmChain::new("BNB", "BNB Smart Chain");
pub const MATIC: EvmChain = EvmChain::new("MATIC", "Polygon");
pub const ARB: EvmChain = EvmChain::new("ARB", "Arbitrum");
pub const OP: EvmChain = EvmChain::new("OP", "Optimism");
pub const AVAX: EvmChain = EvmChain::new("AVAX", "Avalanche C-Chain");
pub const FTM: EvmChain = EvmChain::new("FTM", "Fantom");
pub const GNO: EvmChain = EvmChain::new("GNO", "Gnosis Chain");
pub const CELO: EvmChain = EvmChain::new("CELO", "Celo");
pub const ETC: EvmChain = EvmChain::new("ETC", "Ethereum Classic");
pub const VET: EvmChain = EvmChain::new("VET", "VeChain");
pub const FLR: EvmChain = EvmChain::new("FLR", "Flare");
pub const CRO: EvmChain = EvmChain::new("CRO", "Cronos");
pub const MNT: EvmChain = EvmChain::new("MNT", "Mantle");
pub const IMX: EvmChain = EvmChain::new("IMX", "Immutable");
pub const HYPE: EvmChain = EvmChain::new("HYPE", "Hyperliquid");
pub const MEMECORE: EvmChain = EvmChain::new("MEMECORE", "MemeCore");
pub const MONAD: EvmChain = EvmChain::new("MONAD", "Monad");
pub const IP: EvmChain = EvmChain::new("IP", "Story Protocol");

// EVM Tokens (DeFi)
pub const LINK: EvmChain = EvmChain::new("LINK", "Chainlink");
pub const UNI: EvmChain = EvmChain::new("UNI", "Uniswap");
pub const AAVE: EvmChain = EvmChain::new("AAVE", "Aave");
pub const CRV: EvmChain = EvmChain::new("CRV", "Curve DAO");
pub const LDO: EvmChain = EvmChain::new("LDO", "Lido DAO");
pub const ETHFI: EvmChain = EvmChain::new("ETHFI", "ether.fi");
pub const AERO: EvmChain = EvmChain::new("AERO", "Aerodrome");
pub const MORPHO: EvmChain = EvmChain::new("MORPHO", "Morpho");
pub const ZRO: EvmChain = EvmChain::new("ZRO", "LayerZero");
pub const ONDO: EvmChain = EvmChain::new("ONDO", "Ondo");
pub const CAKE: EvmChain = EvmChain::new("CAKE", "PancakeSwap");
pub const VIRTUAL: EvmChain = EvmChain::new("VIRTUAL", "Virtuals Protocol");
pub const MYX: EvmChain = EvmChain::new("MYX", "MYX Finance");
pub const LIT: EvmChain = EvmChain::new("LIT", "Lighter");

// EVM Tokens (Stablecoins/Gold)
pub const USDT_ERC20: EvmChain = EvmChain::new("USDT", "Tether (ERC-20)");
pub const USDC_ERC20: EvmChain = EvmChain::new("USDC", "USD Coin (ERC-20)");
pub const USDE: EvmChain = EvmChain::new("USDe", "Ethena USDe");
pub const DAI: EvmChain = EvmChain::new("DAI", "Dai");
pub const XAUT: EvmChain = EvmChain::new("XAUt", "Tether Gold");
pub const PAXG: EvmChain = EvmChain::new("PAXG", "PAX Gold");
pub const PYUSD: EvmChain = EvmChain::new("PYUSD", "PayPal USD");
pub const FDUSD: EvmChain = EvmChain::new("FDUSD", "First Digital USD");
pub const TUSD: EvmChain = EvmChain::new("TUSD", "TrueUSD");
pub const USDG: EvmChain = EvmChain::new("USDG", "Global Dollar");
pub const USD1: EvmChain = EvmChain::new("USD1", "World Liberty USD");
pub const RLUSD_ERC20: EvmChain = EvmChain::new("RLUSD", "Ripple USD (ERC-20)");

// EVM Tokens (Exchange)
pub const LEO: EvmChain = EvmChain::new("LEO", "UNUS SED LEO");
pub const BGB: EvmChain = EvmChain::new("BGB", "Bitget Token");
pub const OKB: EvmChain = EvmChain::new("OKB", "OKB");
pub const KCS: EvmChain = EvmChain::new("KCS", "KuCoin Token");
pub const GT: EvmChain = EvmChain::new("GT", "GateToken");
pub const NEXO: EvmChain = EvmChain::new("NEXO", "Nexo");
pub const CHZ: EvmChain = EvmChain::new("CHZ", "Chiliz");

// EVM Tokens (Memes & Other)
pub const SHIB: EvmChain = EvmChain::new("SHIB", "Shiba Inu");
pub const PEPE: EvmChain = EvmChain::new("PEPE", "Pepe");
pub const FLOKI: EvmChain = EvmChain::new("FLOKI", "FLOKI");
pub const WLD: EvmChain = EvmChain::new("WLD", "Worldcoin");
pub const FET: EvmChain = EvmChain::new("FET", "ASI Alliance");
pub const QNT: EvmChain = EvmChain::new("QNT", "Quant");
pub const ENA: EvmChain = EvmChain::new("ENA", "Ethena");
pub const SKY: EvmChain = EvmChain::new("SKY", "Sky");
pub const ASTER: EvmChain = EvmChain::new("ASTER", "Aster");
pub const WLFI: EvmChain = EvmChain::new("WLFI", "World Liberty");
pub const SPX: EvmChain = EvmChain::new("SPX", "SPX6900");
pub const CMC20: EvmChain = EvmChain::new("CMC20", "CMC 20 Index");

// Keep legacy Ethereum struct for backwards compatibility
pub type Ethereum = EvmChain;
pub const ETHEREUM: EvmChain = ETH;

impl Chain for EvmChain {
    fn ticker(&self) -> &'static str {
        self.ticker
    }

    fn name(&self) -> &'static str {
        self.name
    }

    fn family(&self) -> ChainFamily {
        ChainFamily::Evm
    }

    fn address_types(&self) -> Vec<AddressType> {
        vec![AddressType::Evm]
    }

    fn default_address_type(&self) -> AddressType {
        AddressType::Evm
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
        "0123456789abcdefABCDEF"
    }

    fn address_prefix(&self, _address_type: AddressType) -> &'static str {
        "0x"
    }

    fn generate_address(&self, _address_type: AddressType) -> (String, Vec<u8>) {
        let keypair = Secp256k1Keypair::generate();
        let pubkey_xy = keypair.public_key_xy();
        let hash = keccak256(&pubkey_xy);
        
        let mut address_bytes = [0u8; 20];
        address_bytes.copy_from_slice(&hash[12..32]);
        
        let address = eip55_checksum(&address_bytes);
        (address, keypair.private_key_bytes().to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eth_generation() {
        let addr = ETH.generate(AddressType::Evm);
        assert!(addr.address.starts_with("0x"));
        assert_eq!(addr.address.len(), 42);
        assert_eq!(addr.chain, "ETH");
    }

    #[test]
    fn test_bnb_generation() {
        let addr = BNB.generate(AddressType::Evm);
        assert!(addr.address.starts_with("0x"));
        assert_eq!(addr.chain, "BNB");
    }

    #[test]
    fn test_matic_generation() {
        let addr = MATIC.generate(AddressType::Evm);
        assert!(addr.address.starts_with("0x"));
        assert_eq!(addr.chain, "MATIC");
    }

    #[test]
    fn test_known_vector() {
        // Private key = 1
        let privkey = hex::decode("0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        let addr = ETH.generate_from_bytes(&privkey, AddressType::Evm).unwrap();
        
        // Known address for privkey=1
        assert_eq!(addr.address.to_lowercase(), "0x7e5f4552091a69125d5dfcb7b8c2659029395bdf");
    }
}
