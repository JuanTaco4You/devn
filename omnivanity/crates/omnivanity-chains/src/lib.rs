//! OmniVanity Chain Adapters
//!
//! Trait-based abstraction for multi-chain vanity address generation.
//! Supports 50+ chains across multiple address families.

pub mod traits;

// Chain adapter modules
pub mod ethereum;
pub mod bitcoin;
pub mod solana;
pub mod litecoin;
pub mod dogecoin;
pub mod zcash;
pub mod cosmos;
pub mod dash;
pub mod ravencoin;
pub mod digibyte;
pub mod tron;
pub mod xrp;
pub mod stellar;
pub mod aptos;
pub mod sui;
pub mod near;
pub mod iota;
pub mod algorand;
pub mod polkadot;
pub mod filecoin;
pub mod zilliqa;
pub mod nano;

// Re-exports
pub use traits::{Chain, ChainFamily, AddressType, GeneratedAddress};

// EVM chains
pub use ethereum::{EvmChain, ETH, BNB, MATIC, ARB, OP, AVAX, FTM, GNO, CELO};

// UTXO chains
pub use bitcoin::Bitcoin;
pub use litecoin::Litecoin;
pub use dogecoin::Dogecoin;
pub use zcash::Zcash;
pub use dash::Dash;
pub use ravencoin::Ravencoin;
pub use digibyte::Digibyte;

// Cosmos chains
pub use cosmos::{CosmosChain, ATOM, OSMO, INJ, SEI, TIA, JUNO, KAVA, SCRT, RUNE, CRO};

// Other chains
pub use solana::Solana;
pub use tron::Tron;
pub use xrp::Xrp;
pub use stellar::Stellar;
pub use aptos::Aptos;
pub use sui::Sui;
pub use near::Near;
pub use iota::Iota;
pub use algorand::Algorand;
pub use filecoin::Filecoin;
pub use zilliqa::Zilliqa;
pub use nano::Nano;

// SS58/Polkadot chains
pub use polkadot::{Ss58Chain, DOT, KSM, ACA, CFG, HDX};

/// Get all supported chains (50 total)
pub fn all_chains() -> Vec<Box<dyn Chain>> {
    vec![
        // EVM chains (9)
        Box::new(ETH),
        Box::new(BNB),
        Box::new(MATIC),
        Box::new(ARB),
        Box::new(OP),
        Box::new(AVAX),
        Box::new(FTM),
        Box::new(GNO),
        Box::new(CELO),
        // UTXO chains (7)
        Box::new(Bitcoin),
        Box::new(Litecoin),
        Box::new(Dogecoin),
        Box::new(Zcash),
        Box::new(Dash),
        Box::new(Ravencoin),
        Box::new(Digibyte),
        // Cosmos chains (10)
        Box::new(ATOM),
        Box::new(OSMO),
        Box::new(INJ),
        Box::new(SEI),
        Box::new(TIA),
        Box::new(JUNO),
        Box::new(KAVA),
        Box::new(SCRT),
        Box::new(RUNE),
        Box::new(CRO),
        // Solana
        Box::new(Solana),
        // Other specialized chains
        Box::new(Tron),
        Box::new(Xrp),
        Box::new(Stellar),
        Box::new(Aptos),
        Box::new(Sui),
        Box::new(Near),
        Box::new(Iota),
        Box::new(Algorand),
        Box::new(Filecoin),
        Box::new(Zilliqa),
        Box::new(Nano),
        // SS58/Polkadot chains (5)
        Box::new(DOT),
        Box::new(KSM),
        Box::new(ACA),
        Box::new(CFG),
        Box::new(HDX),
    ]
}

/// Get a chain by ticker
pub fn get_chain(ticker: &str) -> Option<Box<dyn Chain>> {
    match ticker.to_uppercase().as_str() {
        // EVM chains
        "ETH" => Some(Box::new(ETH)),
        "BNB" => Some(Box::new(BNB)),
        "MATIC" | "POL" => Some(Box::new(MATIC)),
        "ARB" => Some(Box::new(ARB)),
        "OP" => Some(Box::new(OP)),
        "AVAX" => Some(Box::new(AVAX)),
        "FTM" => Some(Box::new(FTM)),
        "GNO" => Some(Box::new(GNO)),
        "CELO" => Some(Box::new(CELO)),
        // UTXO chains
        "BTC" => Some(Box::new(Bitcoin)),
        "LTC" => Some(Box::new(Litecoin)),
        "DOGE" => Some(Box::new(Dogecoin)),
        "ZEC" => Some(Box::new(Zcash)),
        "DASH" => Some(Box::new(Dash)),
        "RVN" => Some(Box::new(Ravencoin)),
        "DGB" => Some(Box::new(Digibyte)),
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
        // Solana
        "SOL" => Some(Box::new(Solana)),
        // Other specialized chains
        "TRX" => Some(Box::new(Tron)),
        "XRP" => Some(Box::new(Xrp)),
        "XLM" => Some(Box::new(Stellar)),
        "APT" => Some(Box::new(Aptos)),
        "SUI" => Some(Box::new(Sui)),
        "NEAR" => Some(Box::new(Near)),
        "IOTA" => Some(Box::new(Iota)),
        "ALGO" => Some(Box::new(Algorand)),
        "FIL" => Some(Box::new(Filecoin)),
        "ZIL" => Some(Box::new(Zilliqa)),
        "XNO" | "NANO" => Some(Box::new(Nano)),
        // SS58/Polkadot chains
        "DOT" => Some(Box::new(DOT)),
        "KSM" => Some(Box::new(KSM)),
        "ACA" => Some(Box::new(ACA)),
        "CFG" => Some(Box::new(CFG)),
        "HDX" => Some(Box::new(HDX)),
        _ => None,
    }
}
