//! OmniVanity Tauri Backend
//! 
//! Provides Tauri commands for the GUI frontend to interact with the vanity search engine.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
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

/// Search vanity address
#[tauri::command]
async fn search_vanity(
    chain: String,
    pattern: String,
    pattern_type: String,
    case_insensitive: bool,
) -> Result<GuiSearchResult, String> {
    // Reset stop flag
    STOP_FLAG.store(false, Ordering::Relaxed);
    
    // Get chain
    let chain_impl = get_chain(&chain)
        .ok_or_else(|| format!("Unknown chain: {}", chain))?;
    
    let address_type = chain_impl.default_address_type();
    
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
    let valid_chars = chain_impl.valid_address_chars(address_type);
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
        address_type,
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

/// List available chains
#[tauri::command]
fn list_chains() -> Vec<ChainInfo> {
    omnivanity_core::all_chains()
        .iter()
        .map(|c| ChainInfo {
            ticker: c.ticker().to_string(),
            name: c.name().to_string(),
            prefix: c.address_prefix(c.default_address_type()).to_string(),
        })
        .collect()
}

#[derive(Debug, Clone, Serialize)]
pub struct ChainInfo {
    pub ticker: String,
    pub name: String,
    pub prefix: String,
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
