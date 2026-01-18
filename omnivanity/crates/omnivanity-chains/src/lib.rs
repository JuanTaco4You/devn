//! OmniVanity Chain Adapters
//!
//! Trait-based abstraction for multi-chain vanity address generation.

pub mod traits;
pub mod ethereum;
pub mod bitcoin;
pub mod solana;
pub mod litecoin;
pub mod dogecoin;
pub mod zcash;
pub mod cosmos;

pub use traits::{Chain, ChainFamily, AddressType, GeneratedAddress};
pub use ethereum::Ethereum;
pub use bitcoin::Bitcoin;
pub use solana::Solana;
pub use litecoin::Litecoin;
pub use dogecoin::Dogecoin;
pub use zcash::Zcash;
pub use cosmos::{CosmosChain, ATOM, OSMO, INJ, SEI, TIA, JUNO, KAVA, SCRT, RUNE, CRO};

/// Get all supported chains
pub fn all_chains() -> Vec<Box<dyn Chain>> {
    vec![
        Box::new(Ethereum),
        Box::new(Bitcoin),
        Box::new(Solana),
        Box::new(Litecoin),
        Box::new(Dogecoin),
        Box::new(Zcash),
        // Cosmos chains
        Box::new(ATOM),
        Box::new(OSMO),
        Box::new(INJ),
        Box::new(SEI),
        Box::new(TIA),
    ]
}

/// Get a chain by ticker
pub fn get_chain(ticker: &str) -> Option<Box<dyn Chain>> {
    match ticker.to_uppercase().as_str() {
        "ETH" => Some(Box::new(Ethereum)),
        "BTC" => Some(Box::new(Bitcoin)),
        "SOL" => Some(Box::new(Solana)),
        "LTC" => Some(Box::new(Litecoin)),
        "DOGE" => Some(Box::new(Dogecoin)),
        "ZEC" => Some(Box::new(Zcash)),
        // Cosmos chains
        "ATOM" => Some(Box::new(ATOM)),
        "OSMO" => Some(Box::new(OSMO)),
        "INJ" => Some(Box::new(INJ)),
        "SEI" => Some(Box::new(SEI)),
        "TIA" => Some(Box::new(TIA)),
        "JUNO" => Some(Box::new(JUNO)),
        "KAVA" => Some(Box::new(KAVA)),
        "SCRT" => Some(Box::new(SCRT)),
        "RUNE" => Some(Box::new(RUNE)),
        "CRO" => Some(Box::new(CRO)),
        _ => None,
    }
}
