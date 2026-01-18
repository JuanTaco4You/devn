//! OmniVanity Tauri Backend
//! 
//! Provides Tauri commands for the GUI frontend to interact with the vanity search engine.

use std::sync::atomic::{AtomicBool, Ordering};
use serde::{Deserialize, Serialize};
use omnivanity_core::{
    VanitySearch, SearchConfig, Pattern, PatternType, 
    AddressType, get_chain,
};

// Global stop flag for cancellation
static STOP_FLAG: AtomicBool = AtomicBool::new(false);

/// Search result for GUI
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GuiSearchResult {
    pub address: String,
    pub private_key_hex: String,
    pub private_key_native: String,
    pub public_key_hex: String,
    pub keys_tested_formatted: String,
    pub time_secs: f64,
    pub keys_per_second: f64,
}

#[tauri::command]
async fn search_vanity(
    chain: String,
    pattern: String,
    pattern_type: String,
    case_insensitive: bool,
    address_type: Option<String>,
) -> Result<GuiSearchResult, String> {
    // Reset stop flag
    STOP_FLAG.store(false, Ordering::Relaxed);
    
    // Get chain
    let chain_impl = get_chain(&chain)
        .ok_or_else(|| format!("Unknown chain: {}", chain))?;
    
    // Parse address type (default to chain's default)
    let addr_type = match address_type.as_deref() {
        Some("legacy") | Some("p2pkh") => AddressType::P2pkh,
        Some("segwit") | Some("p2wpkh") => AddressType::P2wpkh,
        Some("taproot") | Some("p2tr") => AddressType::P2tr,
        _ => chain_impl.default_address_type(),
    };
    
    // Parse pattern type
    let pat_type = match pattern_type.as_str() {
        "prefix" => PatternType::Prefix,
        "suffix" => PatternType::Suffix,
        "contains" => PatternType::Contains,
        _ => PatternType::Prefix,
    };
    
    // Create pattern
    let mut pat = Pattern {
        value: pattern.clone(),
        pattern_type: pat_type,
        case_insensitive,
    };
    
    // Validate pattern
    let valid_chars = chain_impl.valid_address_chars(addr_type);
    pat.validate(valid_chars)
        .map_err(|e| format!("Invalid pattern: {}", e))?;
    
    // Create search config
    let config = SearchConfig {
        threads: 0, // Auto
        batch_size: 1000,
        max_attempts: 0,
        max_time_secs: 300, // 5 min max for GUI
    };
    
    // Create and run search
    let search = VanitySearch::new(
        chain_impl,
        addr_type,
        vec![pat],
        config,
    );
    
    // Run search in blocking task
    let result = tokio::task::spawn_blocking(move || {
        search.run()
    }).await.map_err(|e| format!("Search task failed: {}", e))?;
    
    match result {
        Some(r) => Ok(GuiSearchResult {
            address: r.address.address,
            private_key_hex: r.address.private_key_hex,
            private_key_native: r.address.private_key_native,
            public_key_hex: r.address.public_key_hex,
            keys_tested_formatted: format_keys(r.keys_tested),
            time_secs: r.time_secs,
            keys_per_second: r.keys_per_second,
        }),
        None => Err("No match found within time limit".to_string()),
    }
}

/// Stop ongoing search
#[tauri::command]
fn stop_search() {
    STOP_FLAG.store(true, Ordering::Relaxed);
}

#[tauri::command]
fn list_chains() -> Vec<ChainInfo> {
    omnivanity_core::all_chains()
        .iter()
        .map(|c| {
            let default_type = c.default_address_type();
            ChainInfo {
                ticker: c.ticker().to_string(),
                name: c.name().to_string(),
                prefix: c.address_prefix(default_type).to_string(),
                address_types: c.address_types().into_iter().map(|at| {
                    let (id, name) = match at {
                        AddressType::P2pkh => ("legacy", "Legacy (1...)"),
                        AddressType::P2wpkh => ("segwit", "SegWit (bc1q...)"),
                        AddressType::P2tr => ("taproot", "Taproot (bc1p...)"),
                        AddressType::Evm => ("evm", "EVM (0x...)"),
                        AddressType::Solana => ("solana", "Solana"),
                        AddressType::P2sh => ("p2sh", "P2SH (3...)"),
                        AddressType::Cosmos => ("cosmos", "Cosmos Bech32"),
                        AddressType::Tron => ("tron", "TRON"),
                        AddressType::Xrpl => ("xrpl", "XRP Ledger"),
                        AddressType::Stellar => ("stellar", "Stellar"),
                        AddressType::Aptos => ("aptos", "Aptos"),
                        AddressType::Sui => ("sui", "Sui"),
                        AddressType::Near => ("near", "NEAR"),
                        AddressType::Iota => ("iota", "IOTA"),
                        AddressType::Algorand => ("algorand", "Algorand"),
                        AddressType::Ss58 => ("ss58", "SS58"),
                        AddressType::Filecoin => ("filecoin", "Filecoin"),
                        AddressType::Zilliqa => ("zilliqa", "Zilliqa"),
                        AddressType::Nano => ("nano", "Nano"),
                        AddressType::Ton => ("ton", "TON"),
                        AddressType::Stacks => ("stacks", "Stacks"),
                        AddressType::Xdc => ("xdc", "XDC Network"),
                        AddressType::Midnight => ("midnight", "Midnight"),
                        AddressType::Kaspa => ("kaspa", "Kaspa"),
                        AddressType::Tezos => ("tezos", "Tezos"),
                        AddressType::CashAddr => ("cashaddr", "CashAddr"),
                        AddressType::Cardano => ("cardano", "Cardano"),
                    };
                    AddressTypeInfo {
                        id: id.to_string(),
                        name: name.to_string(),
                        prefix: c.address_prefix(at).to_string(),
                        is_default: at == default_type,
                    }
                }).collect(),
            }
        })
        .collect()
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainInfo {
    pub ticker: String,
    pub name: String,
    pub prefix: String,
    pub address_types: Vec<AddressTypeInfo>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddressTypeInfo {
    pub id: String,
    pub name: String,
    pub prefix: String,
    pub is_default: bool,
}

fn format_keys(keys: u64) -> String {
    if keys >= 1_000_000_000 {
        format!("{:.2}B", keys as f64 / 1e9)
    } else if keys >= 1_000_000 {
        format!("{:.2}M", keys as f64 / 1e6)
    } else if keys >= 1000 {
        format!("{:.1}K", keys as f64 / 1e3)
    } else {
        format!("{}", keys)
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            search_vanity,
            stop_search,
            list_chains
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
