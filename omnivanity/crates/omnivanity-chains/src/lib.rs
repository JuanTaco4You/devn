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

pub use traits::{Chain, ChainFamily, AddressType, GeneratedAddress};
pub use ethereum::Ethereum;
pub use bitcoin::Bitcoin;
pub use solana::Solana;
pub use litecoin::Litecoin;
pub use dogecoin::Dogecoin;
pub use zcash::Zcash;

/// Get all supported chains
pub fn all_chains() -> Vec<Box<dyn Chain>> {
    vec![
        Box::new(Ethereum),
        Box::new(Bitcoin),
        Box::new(Solana),
        Box::new(Litecoin),
        Box::new(Dogecoin),
        Box::new(Zcash),
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
        _ => None,
    }
}
