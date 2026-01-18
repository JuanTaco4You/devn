//! OmniVanity Chain Adapters
//!
//! Trait-based abstraction for multi-chain vanity address generation.
//! Supports 110+ chains/tokens across multiple address families.

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
pub mod ton;
pub mod stacks;
pub mod xdc;
pub mod midnight;
pub mod kaspa;
pub mod tezos;
pub mod bch;
pub mod cardano;
pub mod monero;
pub mod hedera;
pub mod icp;

// Re-exports
pub use traits::{Chain, ChainFamily, AddressType, GeneratedAddress};

// EVM chains and tokens (60+)
pub use ethereum::{
    EvmChain, ETH, BNB, MATIC, ARB, OP, AVAX, FTM, GNO, CELO,
    ETC, VET, FLR, CRO, MNT, IMX, HYPE, MEMECORE, MONAD, IP,
    LINK, UNI, AAVE, CRV, LDO, ETHFI, AERO, MORPHO, ZRO, ONDO, CAKE, VIRTUAL, MYX, LIT,
    USDT_ERC20, USDC_ERC20, USDE, DAI, XAUT, PAXG, PYUSD, FDUSD, TUSD, USDG, USD1, RLUSD_ERC20,
    LEO, BGB, OKB, KCS, GT, NEXO, CHZ,
    SHIB, PEPE, FLOKI, WLD, FET, QNT, ENA, SKY, ASTER, WLFI, SPX, CMC20,
};

// Solana chains and tokens
pub use solana::{SolanaChain, SOL, TRUMP, BONK, PENGU, PUMP, JUP, RENDER, USDT_SPL, USDC_SPL};

// TRON chains and tokens
pub use tron::{TronChain, TRX, USDT_TRC20, USDC_TRC20, USDD};

// UTXO chains
pub use bitcoin::Bitcoin;
pub use litecoin::Litecoin;
pub use dogecoin::Dogecoin;
pub use zcash::Zcash;
pub use dash::Dash;
pub use ravencoin::Ravencoin;
pub use digibyte::Digibyte;
pub use bch::BitcoinCash;

// Cosmos chains
pub use cosmos::{CosmosChain, ATOM, OSMO, INJ, SEI, TIA, JUNO, KAVA, SCRT, RUNE};

// Other chains
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
pub use ton::Ton;
pub use stacks::Stacks;
pub use xdc::Xdc;
pub use midnight::Midnight;
pub use kaspa::Kaspa;
pub use tezos::Tezos;
pub use cardano::Cardano;
pub use monero::Monero;
pub use hedera::Hedera;
pub use icp::Icp;

// SS58/Polkadot chains
pub use polkadot::{Ss58Chain, DOT, KSM, ACA, CFG, HDX};

/// Get all supported chains (110+)
pub fn all_chains() -> Vec<Box<dyn Chain>> {
    vec![
        // EVM chains and tokens
        Box::new(ETH), Box::new(BNB), Box::new(MATIC), Box::new(ARB), Box::new(OP),
        Box::new(AVAX), Box::new(FTM), Box::new(GNO), Box::new(CELO), Box::new(ETC),
        Box::new(VET), Box::new(FLR), Box::new(CRO), Box::new(MNT), Box::new(IMX),
        Box::new(HYPE), Box::new(MEMECORE), Box::new(MONAD), Box::new(IP),
        Box::new(LINK), Box::new(UNI), Box::new(AAVE), Box::new(CRV), Box::new(LDO),
        Box::new(ETHFI), Box::new(AERO), Box::new(MORPHO), Box::new(ZRO), Box::new(ONDO),
        Box::new(CAKE), Box::new(VIRTUAL), Box::new(MYX), Box::new(LIT),
        Box::new(USDT_ERC20), Box::new(USDC_ERC20), Box::new(USDE), Box::new(DAI),
        Box::new(XAUT), Box::new(PAXG), Box::new(PYUSD), Box::new(FDUSD), Box::new(TUSD),
        Box::new(USDG), Box::new(USD1), Box::new(RLUSD_ERC20),
        Box::new(LEO), Box::new(BGB), Box::new(OKB), Box::new(KCS), Box::new(GT),
        Box::new(NEXO), Box::new(CHZ),
        Box::new(SHIB), Box::new(PEPE), Box::new(FLOKI), Box::new(WLD), Box::new(FET),
        Box::new(QNT), Box::new(ENA), Box::new(SKY), Box::new(ASTER), Box::new(WLFI),
        Box::new(SPX), Box::new(CMC20),
        // Solana chains and tokens
        Box::new(SOL), Box::new(TRUMP), Box::new(BONK), Box::new(PENGU), Box::new(PUMP),
        Box::new(JUP), Box::new(RENDER), Box::new(USDT_SPL), Box::new(USDC_SPL),
        // TRON chains and tokens
        Box::new(TRX), Box::new(USDT_TRC20), Box::new(USDC_TRC20), Box::new(USDD),
        // UTXO chains
        Box::new(Bitcoin), Box::new(Litecoin), Box::new(Dogecoin), Box::new(Zcash),
        Box::new(Dash), Box::new(Ravencoin), Box::new(Digibyte), Box::new(BitcoinCash),
        // Cosmos chains
        Box::new(ATOM), Box::new(OSMO), Box::new(INJ), Box::new(SEI), Box::new(TIA),
        Box::new(JUNO), Box::new(KAVA), Box::new(SCRT), Box::new(RUNE),
        // Other chains
        Box::new(Xrp), Box::new(Stellar), Box::new(Aptos), Box::new(Sui),
        Box::new(Near), Box::new(Iota), Box::new(Algorand), Box::new(Filecoin),
        Box::new(Zilliqa), Box::new(Nano), Box::new(Ton), Box::new(Stacks), Box::new(Xdc),
        Box::new(Midnight), Box::new(Kaspa), Box::new(Tezos), Box::new(Cardano), Box::new(Monero),
        Box::new(Hedera), Box::new(Icp),
        // SS58/Polkadot chains
        Box::new(DOT), Box::new(KSM), Box::new(ACA), Box::new(CFG), Box::new(HDX),
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
        "ETC" => Some(Box::new(ETC)),
        "VET" => Some(Box::new(VET)),
        "FLR" => Some(Box::new(FLR)),
        "CRO" => Some(Box::new(CRO)),
        "MNT" => Some(Box::new(MNT)),
        "IMX" => Some(Box::new(IMX)),
        "HYPE" => Some(Box::new(HYPE)),
        "MEMECORE" => Some(Box::new(MEMECORE)),
        "MONAD" => Some(Box::new(MONAD)),
        "IP" => Some(Box::new(IP)),
        // EVM tokens
        "LINK" => Some(Box::new(LINK)),
        "UNI" => Some(Box::new(UNI)),
        "AAVE" => Some(Box::new(AAVE)),
        "CRV" => Some(Box::new(CRV)),
        "LDO" => Some(Box::new(LDO)),
        "ETHFI" => Some(Box::new(ETHFI)),
        "AERO" => Some(Box::new(AERO)),
        "MORPHO" => Some(Box::new(MORPHO)),
        "ZRO" => Some(Box::new(ZRO)),
        "ONDO" => Some(Box::new(ONDO)),
        "CAKE" => Some(Box::new(CAKE)),
        "VIRTUAL" => Some(Box::new(VIRTUAL)),
        "MYX" => Some(Box::new(MYX)),
        "LIT" => Some(Box::new(LIT)),
        "USDT" | "USDT-ERC20" => Some(Box::new(USDT_ERC20)),
        "USDC" | "USDC-ERC20" => Some(Box::new(USDC_ERC20)),
        "USDE" => Some(Box::new(USDE)),
        "DAI" => Some(Box::new(DAI)),
        "XAUT" => Some(Box::new(XAUT)),
        "PAXG" => Some(Box::new(PAXG)),
        "PYUSD" => Some(Box::new(PYUSD)),
        "FDUSD" => Some(Box::new(FDUSD)),
        "TUSD" => Some(Box::new(TUSD)),
        "USDG" => Some(Box::new(USDG)),
        "USD1" => Some(Box::new(USD1)),
        "RLUSD" | "RLUSD-ERC20" => Some(Box::new(RLUSD_ERC20)),
        "LEO" => Some(Box::new(LEO)),
        "BGB" => Some(Box::new(BGB)),
        "OKB" => Some(Box::new(OKB)),
        "KCS" => Some(Box::new(KCS)),
        "GT" => Some(Box::new(GT)),
        "NEXO" => Some(Box::new(NEXO)),
        "CHZ" => Some(Box::new(CHZ)),
        "SHIB" => Some(Box::new(SHIB)),
        "PEPE" => Some(Box::new(PEPE)),
        "FLOKI" => Some(Box::new(FLOKI)),
        "WLD" => Some(Box::new(WLD)),
        "FET" => Some(Box::new(FET)),
        "QNT" => Some(Box::new(QNT)),
        "ENA" => Some(Box::new(ENA)),
        "SKY" => Some(Box::new(SKY)),
        "ASTER" => Some(Box::new(ASTER)),
        "WLFI" => Some(Box::new(WLFI)),
        "SPX" => Some(Box::new(SPX)),
        "CMC20" => Some(Box::new(CMC20)),
        // Solana chains and tokens
        "SOL" => Some(Box::new(SOL)),
        "TRUMP" => Some(Box::new(TRUMP)),
        "BONK" => Some(Box::new(BONK)),
        "PENGU" => Some(Box::new(PENGU)),
        "PUMP" => Some(Box::new(PUMP)),
        "JUP" => Some(Box::new(JUP)),
        "RENDER" => Some(Box::new(RENDER)),
        "USDT-SPL" => Some(Box::new(USDT_SPL)),
        "USDC-SPL" => Some(Box::new(USDC_SPL)),
        // TRON chains and tokens
        "TRX" => Some(Box::new(TRX)),
        "USDT-TRC20" => Some(Box::new(USDT_TRC20)),
        "USDC-TRC20" => Some(Box::new(USDC_TRC20)),
        "USDD" => Some(Box::new(USDD)),
        // UTXO chains
        "BTC" => Some(Box::new(Bitcoin)),
        "LTC" => Some(Box::new(Litecoin)),
        "DOGE" => Some(Box::new(Dogecoin)),
        "ZEC" => Some(Box::new(Zcash)),
        "DASH" => Some(Box::new(Dash)),
        "RVN" => Some(Box::new(Ravencoin)),
        "DGB" => Some(Box::new(Digibyte)),
        "BCH" => Some(Box::new(BitcoinCash)),
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
        // Other chains
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
        "TON" => Some(Box::new(Ton)),
        "STX" => Some(Box::new(Stacks)),
        "XDC" => Some(Box::new(Xdc)),
        "NIGHT" | "MIDNIGHT" => Some(Box::new(Midnight)),
        "KAS" => Some(Box::new(Kaspa)),
        "XTZ" => Some(Box::new(Tezos)),
        "ADA" => Some(Box::new(Cardano)),
        "XMR" => Some(Box::new(Monero)),
        "HBAR" => Some(Box::new(Hedera)),
        "ICP" => Some(Box::new(Icp)),
        // SS58/Polkadot chains
        "DOT" => Some(Box::new(DOT)),
        "KSM" => Some(Box::new(KSM)),
        "ACA" => Some(Box::new(ACA)),
        "CFG" => Some(Box::new(CFG)),
        "HDX" => Some(Box::new(HDX)),
        // Substrate-based
        "TAO" => Some(Box::new(DOT)), // Bittensor uses similar to Polkadot
        _ => None,
    }
}
