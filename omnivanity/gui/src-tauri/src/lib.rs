//! OmniVanity Tauri Backend
//! 
//! Provides Tauri commands for the GUI frontend to interact with the vanity search engine.

use std::sync::atomic::{AtomicBool, Ordering};
use serde::{Deserialize, Serialize};
use omnivanity_core::{
    VanitySearch, SearchConfig, Pattern, PatternType, 
    AddressType, get_chain,
};

use tauri::{AppHandle, Emitter};

// Global stop flag for cancellation
static STOP_FLAG: AtomicBool = AtomicBool::new(false);

/// Real-time search statistics event
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SearchStatsEvent {
    keys_tested: String,
    keys_per_second: f64,
    keys_per_second_fmt: String,
    probability_percent: f64,
    est_time_50_percent: String,
}

/// Log message event
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SearchLogEvent {
    timestamp: String,
    level: String, // "info", "success", "error"
    message: String,
}

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
    app: AppHandle,
    chain: String,
    pattern: String,
    pattern_type: String,
    case_insensitive: bool,
    address_type: Option<String>,
    use_gpu: Option<bool>,
    batch_size: Option<u32>,
) -> Result<GuiSearchResult, String> {
    // Reset stop flag
    STOP_FLAG.store(false, Ordering::Relaxed);
    
    // Emit initial log
    let _ = app.emit("search-log", SearchLogEvent {
        timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
        level: "info".to_string(),
        message: format!("Starting search for '{}' on {}...", pattern, chain),
    });
    
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
    if let Err(e) = pat.validate(valid_chars) {
        let msg = format!("Invalid pattern: {}", e);
        let _ = app.emit("search-log", SearchLogEvent {
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
            level: "error".to_string(),
            message: msg.clone(),
        });
        return Err(msg);
    }
    
    // Create search config
    let config = SearchConfig {
        threads: 0, // Auto
        // Massive batch size for GPU mode to keep GPU saturated
        // CPU will generate in parallel while GPU processes previous batch
        batch_size: batch_size.unwrap_or(524288) as usize, // 512K default (was 65K)
        max_attempts: 0,
        max_time_secs: 0, // No limit
        use_gpu: use_gpu.unwrap_or(true),
    };
    
    // Create and run search
    let search = VanitySearch::new(
        chain_impl,
        addr_type,
        vec![pat],
        config,
    );
    
    let difficulty = search.difficulty();
    
    // Clone app handle for the blocking task
    let app_handle = app.clone();

    // Run search in blocking task with callback
    let result = tokio::task::spawn_blocking(move || {
        search.run_with_callback(|stats| {
            // Check stop flag
            if STOP_FLAG.load(Ordering::Relaxed) {
                stats.stop();
                return;
            }
            
            // Calculate stats
            let kps = stats.keys_per_second();
            let keys = stats.total_keys();
            
            // Prob calculation
            let prob = if difficulty > 0.0 {
                1.0 - (-1.0 * keys as f64 / difficulty).exp()
            } else {
                0.0
            };

            // ETA 50%
            let remaining_time = if prob < 0.5 && kps > 0.0 {
                let keys_needed = (0.5_f64.ln() / (-1.0 / difficulty)) - keys as f64;
                if keys_needed > 0.0 {
                    format_duration(keys_needed / kps)
                } else {
                    "very soon".to_string()
                }
            } else if prob >= 0.5 {
                "any moment".to_string()
            } else {
                "calculating...".to_string()
            };

            let _ = app_handle.emit("search-stats", SearchStatsEvent {
                keys_tested: format_keys(keys),
                keys_per_second: kps,
                keys_per_second_fmt: format!("{} keys/s", format_keys_short(kps as u64)),
                probability_percent: prob * 100.0,
                est_time_50_percent: remaining_time,
            });
        })
    }).await.map_err(|e| format!("Search task failed: {}", e))?;
    
    match result {
        Some(r) => {
            let _ = app.emit("search-log", SearchLogEvent {
                timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                level: "success".to_string(),
                message: format!("✅ Found match! {}...", &r.address.address[0..std::cmp::min(10, r.address.address.len())]),
            });
            Ok(GuiSearchResult {
                address: r.address.address,
                private_key_hex: r.address.private_key_hex,
                private_key_native: r.address.private_key_native,
                public_key_hex: r.address.public_key_hex,
                keys_tested_formatted: format_keys(r.keys_tested),
                time_secs: r.time_secs,
                keys_per_second: r.keys_per_second,
            })
        },
        None => {
            let msg = if STOP_FLAG.load(Ordering::Relaxed) {
                "Search stopped by user"
            } else {
                "No match found within limits"
            };
            let _ = app.emit("search-log", SearchLogEvent {
                timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                level: "info".to_string(),
                message: msg.to_string(),
            });
            Err(msg.to_string())
        },
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
                        AddressType::Monero => ("monero", "Monero"),
                        AddressType::Icp => ("icp", "ICP Principal"),
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

fn format_keys_short(keys: u64) -> String {
    if keys >= 1_000_000_000 {
        format!("{:.1}B", keys as f64 / 1e9)
    } else if keys >= 1_000_000 {
        format!("{:.1}M", keys as f64 / 1e6)
    } else if keys >= 1000 {
        format!("{:.1}K", keys as f64 / 1e3)
    } else {
        format!("{}", keys)
    }
}

fn format_duration(seconds: f64) -> String {
    if seconds <= 0.0 {
        return "now".to_string();
    }
    if seconds < 1.0 {
        format!("{:.0}ms", seconds * 1000.0)
    } else if seconds < 60.0 {
        format!("{:.0}s", seconds)
    } else if seconds < 3600.0 {
        format!("{:.0}m", seconds / 60.0)
    } else if seconds < 86400.0 {
        format!("{:.1}h", seconds / 3600.0)
    } else {
        format!("{:.1}d", seconds / 86400.0)
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
