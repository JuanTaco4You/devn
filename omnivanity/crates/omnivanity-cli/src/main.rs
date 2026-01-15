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

        /// Number of threads (0 = auto)
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
    },

    /// List supported chains
    Chains,

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
        } => {
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
            )?;
        }
        Commands::Chains => {
            cmd_chains();
        }
        Commands::Benchmark {
            chain,
            duration,
            threads,
        } => {
            cmd_benchmark(&chain, duration, threads)?;
        }
    }

    Ok(())
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
        eprintln!("Threads: {}", if threads == 0 { num_cpus::get() } else { threads });
        eprintln!();
    }

    // Create search config
    let config = SearchConfig {
        threads,
        batch_size: 1000,
        max_attempts,
        max_time_secs: max_time,
    };

    // Create search
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

    // Run search
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

fn cmd_benchmark(chain_ticker: &str, duration_secs: u64, threads: usize) -> Result<()> {
    let chain = get_chain(chain_ticker)
        .ok_or_else(|| anyhow::anyhow!("Unknown chain: {}", chain_ticker))?;

    let address_type = chain.default_address_type();

    eprintln!("Benchmarking {} for {} seconds...", chain.name(), duration_secs);
    eprintln!("Threads: {}", if threads == 0 { num_cpus::get() } else { threads });
    eprintln!();

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

    // Run search (will timeout)
    let _ = search.run();

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
