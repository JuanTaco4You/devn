//! Chain trait and types

use serde::{Deserialize, Serialize};
use std::fmt;

/// Chain family categorization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChainFamily {
    /// EVM-compatible chains (ETH, BSC, Polygon, etc.)
    Evm,
    /// Bitcoin-like UTXO chains (BTC, LTC, DOGE, ZEC t-addr)
    UtxoSecp256k1,
    /// Ed25519-based chains (Solana)
    Ed25519,
}

/// Address type for a chain
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AddressType {
    /// EVM address (0x...)
    Evm,
    /// Legacy P2PKH (1...)
    P2pkh,
    /// P2SH (3...)
    P2sh,
    /// Native SegWit P2WPKH (bc1q...)
    P2wpkh,
    /// Taproot P2TR (bc1p...)
    P2tr,
    /// Solana address (Base58)
    Solana,
    /// Cosmos-SDK Bech32 (cosmos1..., osmo1..., etc.)
    Cosmos,
    /// TRON address (T...)
    Tron,
    /// XRP Ledger address (r...)
    Xrpl,
    /// Stellar StrKey (G...)
    Stellar,
    /// Aptos hex address (0x...)
    Aptos,
    /// Sui hex address (0x...)
    Sui,
    /// NEAR implicit account (64 hex)
    Near,
    /// IOTA hex address (0x...)
    Iota,
    /// Algorand Base32 address
    Algorand,
    /// Polkadot/Substrate SS58 address
    Ss58,
    /// Filecoin address (f1...)
    Filecoin,
    /// Zilliqa Bech32 (zil1...)
    Zilliqa,
    /// Nano address (nano_...)
    Nano,
}

impl fmt::Display for AddressType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AddressType::Evm => write!(f, "EVM"),
            AddressType::P2pkh => write!(f, "P2PKH (Legacy)"),
            AddressType::P2sh => write!(f, "P2SH"),
            AddressType::P2wpkh => write!(f, "P2WPKH (SegWit)"),
            AddressType::P2tr => write!(f, "P2TR (Taproot)"),
            AddressType::Solana => write!(f, "Solana"),
            AddressType::Cosmos => write!(f, "Cosmos Bech32"),
            AddressType::Tron => write!(f, "TRON"),
            AddressType::Xrpl => write!(f, "XRP Ledger"),
            AddressType::Stellar => write!(f, "Stellar StrKey"),
            AddressType::Aptos => write!(f, "Aptos"),
            AddressType::Sui => write!(f, "Sui"),
            AddressType::Near => write!(f, "NEAR"),
            AddressType::Iota => write!(f, "IOTA"),
            AddressType::Algorand => write!(f, "Algorand"),
            AddressType::Ss58 => write!(f, "SS58"),
            AddressType::Filecoin => write!(f, "Filecoin"),
            AddressType::Zilliqa => write!(f, "Zilliqa"),
            AddressType::Nano => write!(f, "Nano"),
        }
    }
}

/// A generated address with its keypair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedAddress {
    /// The address string
    pub address: String,
    /// Private key in hex format
    pub private_key_hex: String,
    /// Private key in chain-native format (WIF, etc.)
    pub private_key_native: String,
    /// Public key in hex format
    pub public_key_hex: String,
    /// Chain ticker
    pub chain: String,
    /// Address type used
    pub address_type: AddressType,
}

/// Trait for chain implementations
pub trait Chain: Send + Sync {
    /// Chain ticker symbol (e.g., "ETH", "BTC")
    fn ticker(&self) -> &'static str;
    
    /// Full chain name
    fn name(&self) -> &'static str;
    
    /// Chain family
    fn family(&self) -> ChainFamily;
    
    /// Supported address types for this chain
    fn address_types(&self) -> Vec<AddressType>;
    
    /// Default address type
    fn default_address_type(&self) -> AddressType;
    
    /// Generate a random address
    fn generate(&self, address_type: AddressType) -> GeneratedAddress;
    
    /// Generate from specific private key bytes
    fn generate_from_bytes(&self, private_key: &[u8], address_type: AddressType) -> Option<GeneratedAddress>;
    
    /// Get valid characters for addresses (for pattern validation)
    fn valid_address_chars(&self, address_type: AddressType) -> &'static str;
    
    /// Get the address prefix (e.g., "0x", "1", "bc1q")
    fn address_prefix(&self, address_type: AddressType) -> &'static str;
}
