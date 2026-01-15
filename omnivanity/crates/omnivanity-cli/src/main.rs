//! OmniVanity CLI
//!
//! Multi-chain vanity wallet address generator.

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use omnivanity_core::{
    VanitySearch, SearchConfig, SearchResult,
    Pattern, PatternType, AddressType,
    all_chains, get_chain,
};
use std::io::Write;

#[cfg(feature = "gpu")]
use omnivanity_gpu::{list_devices, is_gpu_available};

#[derive(Parser)]
#[command(name = "omnivanity")]
#[command(author = "OmniVanity Team")]
#[command(version = "0.1.0")]
#[command(about = "Multi-chain vanity wallet address generator", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a vanity address
    Generate {
        /// Chain ticker (ETH, BTC, SOL, LTC, DOGE, ZEC)
        #[arg(short, long, default_value = "ETH")]
        chain: String,

        /// Pattern to search for
        #[arg(short, long)]
        pattern: String,

        /// Pattern type: prefix, suffix, or contains
        #[arg(short = 't', long, default_value = "prefix")]
        pattern_type: PatternTypeArg,

        /// Address type (chain-specific, e.g., p2pkh, p2wpkh)
        #[arg(short, long)]
        address_type: Option<String>,

        /// Case insensitive search
        #[arg(short = 'i', long)]
        case_insensitive: bool,

        /// Number of threads (0 = auto, ignored with --gpu)
        #[arg(long, default_value = "0")]
        threads: usize,

        /// Maximum attempts (0 = unlimited)
        #[arg(long, default_value = "0")]
        max_attempts: u64,

        /// Maximum time in seconds (0 = unlimited)
        #[arg(long, default_value = "0")]
        max_time: u64,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Use GPU for search (requires CUDA)
        #[arg(long)]
        gpu: bool,

        /// GPU device indices to use (comma-separated, e.g., 0,1)
        #[arg(long)]
        device: Option<String>,
    },

    /// List supported chains
    Chains,

    /// List available GPU devices
    #[cfg(feature = "gpu")]
    GpuList,

    /// Run benchmark
    Benchmark {
        /// Chain ticker
        #[arg(short, long, default_value = "ETH")]
        chain: String,

        /// Duration in seconds
        #[arg(short, long, default_value = "5")]
        duration: u64,

        /// Number of threads (0 = auto)
        #[arg(long, default_value = "0")]
        threads: usize,

        /// Use GPU for benchmark
        #[arg(long)]
        gpu: bool,

        /// GPU device indices (comma-separated)
        #[arg(long)]
        device: Option<String>,
    },
}

#[derive(Clone, ValueEnum)]
enum PatternTypeArg {
    Prefix,
    Suffix,
    Contains,
}

impl From<PatternTypeArg> for PatternType {
    fn from(arg: PatternTypeArg) -> Self {
        match arg {
            PatternTypeArg::Prefix => PatternType::Prefix,
            PatternTypeArg::Suffix => PatternType::Suffix,
            PatternTypeArg::Contains => PatternType::Contains,
        }
    }
}

/// Search mode: CPU or GPU
#[derive(Clone, Copy, Debug)]
enum SearchMode {
    Cpu,
    Gpu,
}

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Generate {
            chain,
            pattern,
            pattern_type,
            address_type,
            case_insensitive,
            threads,
            max_attempts,
            max_time,
            json,
            gpu,
            device,
        } => {
            let mode = if gpu { SearchMode::Gpu } else { SearchMode::Cpu };
            let device_indices = parse_device_indices(device.as_deref());
            
            cmd_generate(
                &chain,
                &pattern,
                pattern_type.into(),
                address_type.as_deref(),
                case_insensitive,
                threads,
                max_attempts,
                max_time,
                json,
                mode,
                device_indices,
            )?;
        }
        Commands::Chains => {
            cmd_chains();
        }
        #[cfg(feature = "gpu")]
        Commands::GpuList => {
            cmd_gpu_list();
        }
        Commands::Benchmark {
            chain,
            duration,
            threads,
            gpu,
            device,
        } => {
            let mode = if gpu { SearchMode::Gpu } else { SearchMode::Cpu };
            let device_indices = parse_device_indices(device.as_deref());
            cmd_benchmark(&chain, duration, threads, mode, device_indices)?;
        }
    }

    Ok(())
}

fn parse_device_indices(device: Option<&str>) -> Vec<usize> {
    match device {
        Some(s) => s
            .split(',')
            .filter_map(|d| d.trim().parse().ok())
            .collect(),
        None => vec![],
    }
}

fn cmd_generate(
    chain_ticker: &str,
    pattern: &str,
    pattern_type: PatternType,
    address_type_str: Option<&str>,
    case_insensitive: bool,
    threads: usize,
    max_attempts: u64,
    max_time: u64,
    json_output: bool,
    mode: SearchMode,
    device_indices: Vec<usize>,
) -> Result<()> {
    // Get chain
    let chain = get_chain(chain_ticker)
        .ok_or_else(|| anyhow::anyhow!("Unknown chain: {}", chain_ticker))?;

    // Determine address type
    let address_type = match address_type_str {
        Some(at) => parse_address_type(at)?,
        None => chain.default_address_type(),
    };

    // Validate pattern
    let valid_chars = chain.valid_address_chars(address_type);
    let mut pat = Pattern {
        value: pattern.to_string(),
        pattern_type,
        case_insensitive,
    };
    pat.validate(valid_chars)?;

    if !json_output {
        eprintln!("OmniVanity v0.1.0");
        eprintln!("Chain: {} ({})", chain.name(), chain.ticker());
        eprintln!("Address Type: {}", address_type);
        eprintln!("Pattern: {} ({:?}{})", 
            pattern, 
            pattern_type,
            if case_insensitive { ", case-insensitive" } else { "" }
        );
        
        match mode {
            SearchMode::Cpu => {
                eprintln!("Mode: CPU");
                eprintln!("Threads: {}", if threads == 0 { num_cpus::get() } else { threads });
            }
            SearchMode::Gpu => {
                eprintln!("Mode: GPU");
                if device_indices.is_empty() {
                    eprintln!("Devices: All available");
                } else {
                    eprintln!("Devices: {:?}", device_indices);
                }
            }
        }
        eprintln!();
    }

    // Run search based on mode
    match mode {
        SearchMode::Cpu => {
            run_cpu_search(chain_ticker, address_type, pat, threads, max_attempts, max_time, json_output)?;
        }
        SearchMode::Gpu => {
            #[cfg(feature = "gpu")]
            {
                run_gpu_search(chain_ticker, address_type, pat, max_attempts, max_time, json_output, device_indices)?;
            }
            #[cfg(not(feature = "gpu"))]
            {
                anyhow::bail!("GPU support not compiled. Rebuild with --features gpu");
            }
        }
    }

    Ok(())
}

fn run_cpu_search(
    chain_ticker: &str,
    address_type: AddressType,
    pat: Pattern,
    threads: usize,
    max_attempts: u64,
    max_time: u64,
    json_output: bool,
) -> Result<()> {
    let chain = get_chain(chain_ticker)
        .ok_or_else(|| anyhow::anyhow!("Unknown chain: {}", chain_ticker))?;
    
    let config = SearchConfig {
        threads,
        batch_size: 1000,
        max_attempts,
        max_time_secs: max_time,
    };

    let search = VanitySearch::new(
        chain,
        address_type,
        vec![pat],
        config,
    );

    if !json_output {
        let difficulty = search.difficulty();
        eprintln!("Difficulty: {:.0}", difficulty);
        eprintln!();
    }

    let result = search.run();

    match result {
        Some(result) => {
            if json_output {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                print_result(&result);
            }
        }
        None => {
            if json_output {
                println!("{{\"error\": \"No match found within limits\"}}");
            } else {
                eprintln!("No match found within limits.");
            }
        }
    }

    Ok(())
}

#[cfg(feature = "gpu")]
fn run_gpu_search(
    chain_ticker: &str,
    _address_type: AddressType,
    _pat: Pattern,
    max_attempts: u64,
    max_time: u64,
    json_output: bool,
    device_indices: Vec<usize>,
) -> Result<()> {
    use omnivanity_gpu::{GpuSearchConfig, is_wgpu_available, list_devices, WgpuEngine};

    if !is_wgpu_available() {
        anyhow::bail!("No GPU found. Use CPU mode instead.");
    }

    // Currently only EVM is supported on GPU
    if chain_ticker != "ETH" {
        anyhow::bail!("GPU search currently only supports ETH/EVM. Use --gpu with -c ETH");
    }

    let config = GpuSearchConfig {
        device_indices: device_indices.clone(),
        grid_size: 0, // auto
        block_size: 256,
        keys_per_thread: 256,
        max_attempts,
        max_time_secs: max_time,
    };

    // List available devices
    if !json_output {
        let devices = list_devices();
        eprintln!("GPU Backend: wgpu (cross-platform)");
        eprintln!("Available devices: {}", devices.len());
        for dev in &devices {
            eprintln!("  [{}] {} ({})", dev.index, dev.name, dev.backend);
        }
        eprintln!();
    }

    // Create wgpu engine
    let device_idx = device_indices.first().copied().unwrap_or(0);
    let engine = WgpuEngine::new_sync(device_idx, config)
        .map_err(|e| anyhow::anyhow!("Failed to create wgpu engine: {}", e))?;

    if !json_output {
        eprintln!("Starting GPU search on: {}", engine.device_name());
        eprintln!("(Full GPU search implementation in progress)");
    }

    // TODO: Implement full GPU search with pattern matching and result extraction
    // For now, run benchmark to verify GPU works
    match engine.benchmark(3) {
        Ok(keys_per_sec) => {
            if !json_output {
                let mkeys = keys_per_sec / 1_000_000.0;
                eprintln!("GPU speed: {:.2} Mkey/s", mkeys);
            }
        }
        Err(e) => {
            eprintln!("GPU benchmark failed: {}", e);
        }
    }
    
    Ok(())
}

fn cmd_chains() {
    println!("Supported Chains:");
    println!("{:-<60}", "");
    println!("{:<8} {:<15} {:<20} {}", "Ticker", "Name", "Address Types", "Default");
    println!("{:-<60}", "");
    
    for chain in all_chains() {
        let types: Vec<String> = chain.address_types()
            .iter()
            .map(|t| format!("{}", t))
            .collect();
        
        println!(
            "{:<8} {:<15} {:<20} {}",
            chain.ticker(),
            chain.name(),
            types.join(", "),
            chain.default_address_type()
        );
    }
}

#[cfg(feature = "gpu")]
fn cmd_gpu_list() {
    println!("Available GPU Devices:");
    println!("{:-<60}", "");
    
    if !is_gpu_available() {
        println!("No CUDA-capable GPUs found.");
        println!("Make sure NVIDIA drivers and CUDA toolkit are installed.");
        return;
    }

    let devices = list_devices();
    if devices.is_empty() {
        println!("No GPU devices detected.");
        return;
    }

    for dev in &devices {
        println!(
            "[{}] {} - {} ({}, {} SMs)",
            dev.index,
            dev.name,
            dev.backend,
            dev.memory_formatted(),
            dev.multiprocessors
        );
    }
    
    println!();
    println!("Use --gpu --device N to select specific devices.");
}

fn cmd_benchmark(
    chain_ticker: &str, 
    duration_secs: u64, 
    threads: usize,
    mode: SearchMode,
    device_indices: Vec<usize>,
) -> Result<()> {
    let chain = get_chain(chain_ticker)
        .ok_or_else(|| anyhow::anyhow!("Unknown chain: {}", chain_ticker))?;

    let address_type = chain.default_address_type();

    eprintln!("Benchmarking {} for {} seconds...", chain.name(), duration_secs);
    
    match mode {
        SearchMode::Cpu => {
            eprintln!("Mode: CPU");
            eprintln!("Threads: {}", if threads == 0 { num_cpus::get() } else { threads });
        }
        SearchMode::Gpu => {
            eprintln!("Mode: GPU");
            if device_indices.is_empty() {
                eprintln!("Devices: All available");
            } else {
                eprintln!("Devices: {:?}", device_indices);
            }
        }
    }
    eprintln!();

    match mode {
        SearchMode::Cpu => {
            // Use an impossible pattern to run until timeout
            let pat = Pattern::prefix("zzzzzzzzzzzzzzzzzzz");
            
            let config = SearchConfig {
                threads,
                batch_size: 1000,
                max_attempts: 0,
                max_time_secs: duration_secs,
            };

            let search = VanitySearch::new(
                chain,
                address_type,
                vec![pat],
                config,
            );

            let _ = search.run();
        }
        SearchMode::Gpu => {
            #[cfg(feature = "gpu")]
            {
                if !is_gpu_available() {
                    anyhow::bail!("No CUDA-capable GPU found.");
                }
                eprintln!("GPU benchmark not yet implemented.");
            }
            #[cfg(not(feature = "gpu"))]
            {
                anyhow::bail!("GPU support not compiled.");
            }
        }
    }

    eprintln!("\nBenchmark complete!");

    Ok(())
}

fn print_result(result: &SearchResult) {
    println!();
    println!("🎉 MATCH FOUND!");
    println!("{:-<60}", "");
    println!("Address:     {}", result.address.address);
    println!("Private Key: {}", result.address.private_key_native);
    println!("Private Hex: {}", result.address.private_key_hex);
    println!("Public Key:  {}", result.address.public_key_hex);
    println!("{:-<60}", "");
    println!("Keys Tested: {}", result.keys_tested);
    println!("Time:        {:.2}s", result.time_secs);
    println!("Speed:       {:.2} Mkey/s", result.keys_per_second / 1_000_000.0);
}

fn parse_address_type(s: &str) -> Result<AddressType> {
    match s.to_lowercase().as_str() {
        "p2pkh" | "legacy" => Ok(AddressType::P2pkh),
        "p2sh" => Ok(AddressType::P2sh),
        "p2wpkh" | "segwit" | "bech32" => Ok(AddressType::P2wpkh),
        "p2tr" | "taproot" => Ok(AddressType::P2tr),
        "evm" | "eth" => Ok(AddressType::Evm),
        "solana" | "sol" => Ok(AddressType::Solana),
        _ => Err(anyhow::anyhow!("Unknown address type: {}", s)),
    }
}
