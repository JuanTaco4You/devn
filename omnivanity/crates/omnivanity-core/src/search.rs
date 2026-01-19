//! Vanity search engine

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crossbeam_channel::{bounded, Receiver, Sender};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::info;

use omnivanity_chains::{Chain, AddressType, GeneratedAddress};
use omnivanity_pattern::{Pattern, PatternMatcher, PatternType, calculate_difficulty};

use crate::stats::SearchStats;

// GPU support (optional feature)
#[cfg(feature = "gpu")]
use omnivanity_gpu::{WgpuEngine, MatchType, GpuSearchConfig, is_gpu_available};

// OpenCL Turbo support for Ed25519 chains (optional feature)
#[cfg(feature = "opencl")]
use omnivanity_gpu::{OpenClEngine, OpenClSearchConfig, is_opencl_available};

use omnivanity_chains::ChainFamily;

/// Search configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    /// Number of threads (0 = auto)
    pub threads: usize,
    /// Batch size per thread iteration
    pub batch_size: usize,
    /// Maximum attempts (0 = unlimited)
    pub max_attempts: u64,
    /// Maximum time in seconds (0 = unlimited)
    pub max_time_secs: u64,
    /// Use GPU acceleration if available
    pub use_gpu: bool,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            threads: 0, // Auto-detect
            batch_size: 1000,
            max_attempts: 0,
            max_time_secs: 0,
            use_gpu: true, // Auto-enable GPU if available
        }
    }
}

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// The matching address
    pub address: GeneratedAddress,
    /// Pattern that was matched
    pub pattern: String,
    /// Total keys tested
    pub keys_tested: u64,
    /// Time taken in seconds
    pub time_secs: f64,
    /// Keys per second achieved
    pub keys_per_second: f64,
}

/// Vanity search engine
pub struct VanitySearch {
    chain: Box<dyn Chain>,
    address_type: AddressType,
    matcher: PatternMatcher,
    config: SearchConfig,
    difficulty: f64,
}

impl VanitySearch {
    /// Create a new vanity search
    pub fn new(
        chain: Box<dyn Chain>,
        address_type: AddressType,
        patterns: Vec<Pattern>,
        config: SearchConfig,
    ) -> Self {
        // Calculate difficulty from first pattern
        let difficulty = if let Some(pattern) = patterns.first() {
            let alphabet_size = chain.valid_address_chars(address_type).len();
            calculate_difficulty(
                &pattern.value,
                pattern.pattern_type,
                alphabet_size,
                pattern.case_insensitive,
            )
        } else {
            1.0
        };

        let matcher = PatternMatcher::new(patterns);

        Self {
            chain,
            address_type,
            matcher,
            config,
            difficulty,
        }
    }

    /// Get the search difficulty
    pub fn difficulty(&self) -> f64 {
        self.difficulty
    }

    /// Run the search (blocking until found or limits reached)
    pub fn run(&self) -> Option<SearchResult> {
        // Check for OpenCL Turbo mode (full GPU key gen) for Ed25519 chains
        #[cfg(feature = "opencl")]
        {
            if self.config.use_gpu && self.chain.family() == ChainFamily::Ed25519 && is_opencl_available() {
                info!("🚀 TURBO MODE: Ed25519 chain detected with OpenCL - using full GPU key generation!");
                return self.run_opencl_turbo();
            }
        }
        
        // Check if GPU should be used (hybrid mode for other chains)
        #[cfg(feature = "gpu")]
        {
            if self.config.use_gpu && is_gpu_available() {
                info!("GPU detected, using hybrid CPU+GPU search");
                return self.run_gpu_hybrid();
            }
        }
        
        // Fall back to CPU-only search
        self.run_cpu()
    }
    
    /// CPU-only search (original implementation)
    fn run_cpu(&self) -> Option<SearchResult> {
        let stats = SearchStats::new();
        let stats_clone = stats.clone();

        // Channel for results
        let (tx, rx): (Sender<GeneratedAddress>, Receiver<GeneratedAddress>) = bounded(1);

        // Spawn stats printer thread
        let stats_for_printer = stats.clone();
        let difficulty = self.difficulty;
        let printer_handle = thread::spawn(move || {
            while stats_for_printer.is_running() {
                eprint!("\r{}", stats_for_printer.format(difficulty));
                thread::sleep(Duration::from_millis(250));
            }
            eprintln!(); // New line after stats
        });

        // Configure thread pool
        let num_threads = if self.config.threads == 0 {
            num_cpus::get()
        } else {
            self.config.threads
        };

        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build()
            .expect("Failed to create thread pool");

        // Run parallel search
        let batch_size = self.config.batch_size;
        let max_attempts = self.config.max_attempts;
        let max_time = self.config.max_time_secs;

        pool.install(|| {
            (0..num_threads).into_par_iter().for_each(|_| {
                let mut local_count = 0u64;
                
                while stats_clone.is_running() {
                    // Check limits
                    if max_attempts > 0 && stats_clone.total_keys() >= max_attempts {
                        stats_clone.stop();
                        break;
                    }
                    if max_time > 0 && stats_clone.elapsed().as_secs() >= max_time {
                        stats_clone.stop();
                        break;
                    }

                    // Generate and check batch
                    for _ in 0..batch_size {
                        let addr = self.chain.generate(self.address_type);
                        
                        if self.matcher.matches(&addr.address).is_some() {
                            // Found a match!
                            let _ = tx.try_send(addr);
                            stats_clone.mark_found();
                            return;
                        }
                        
                        local_count += 1;
                    }

                    // Update stats
                    stats_clone.add_keys(batch_size as u64);
                    local_count = 0;
                }

                // Add any remaining
                if local_count > 0 {
                    stats_clone.add_keys(local_count);
                }
            });
        });

        // Wait for printer thread
        stats.stop();
        let _ = printer_handle.join();

        // Check for result
        if let Ok(address) = rx.try_recv() {
            let pattern = self.matcher.patterns()
                .first()
                .map(|p| p.value.clone())
                .unwrap_or_default();

            Some(SearchResult {
                address,
                pattern,
                keys_tested: stats.total_keys(),
                time_secs: stats.elapsed().as_secs_f64(),
                keys_per_second: stats.keys_per_second(),
            })
        } else {
            None
        }
    }
    
    /// GPU-accelerated hybrid search: CPU generates keys, GPU matches patterns
    #[cfg(feature = "gpu")]
    fn run_gpu_hybrid(&self) -> Option<SearchResult> {
        let stats = SearchStats::new();
        
        // Initialize GPU engine
        let gpu_config = GpuSearchConfig::default();
        let gpu_engine = match WgpuEngine::new_sync(0, gpu_config) {
            Ok(g) => {
                info!("GPU initialized: {}", g.device_name());
                g
            }
            Err(e) => {
                info!("GPU init failed ({}), falling back to CPU", e);
                return self.run_cpu();
            }
        };
        
        // Get pattern info
        let pattern = self.matcher.patterns()
            .first()
            .map(|p| p.value.clone())
            .unwrap_or_default();
            
        let pat_obj = self.matcher.patterns().first().unwrap();
        let match_type = match pat_obj.pattern_type {
            PatternType::Prefix => MatchType::Prefix,
            PatternType::Suffix => MatchType::Suffix,
            PatternType::Contains => MatchType::Contains,
        };
        
        // Batch size for GPU (larger = more GPU utilization)
        let gpu_batch_size = self.config.batch_size;
        let max_time = self.config.max_time_secs;
        let max_attempts = self.config.max_attempts;
        let start_time = std::time::Instant::now();
        
        // Stats printer thread  
        let stats_for_printer = stats.clone();
        let difficulty = self.difficulty;
        let printer_handle = thread::spawn(move || {
            while stats_for_printer.is_running() {
                eprint!("\r{} 🚀GPU", stats_for_printer.format(difficulty));
                thread::sleep(Duration::from_millis(250));
            }
            eprintln!();
        });
        
        let mut result: Option<SearchResult> = None;
        
        while stats.is_running() {
            // Check limits
            if max_attempts > 0 && stats.total_keys() >= max_attempts {
                break;
            }
            if max_time > 0 && stats.elapsed().as_secs() >= max_time {
                break;
            }
            
            // Generate batch of addresses on CPU (using rayon with limited threads)
            // Optimization: Use generate_address to avoid full string formatting until match found
            // Use only 75% of CPU cores to avoid maxing out CPU (leave room for GPU driver, system, etc.)
            let num_cpus = num_cpus::get();
            let gen_threads = (num_cpus * 3 / 4).max(1); // Use 75% of cores, minimum 1
            
            let (address_strings, keys): (Vec<String>, Vec<Vec<u8>>) = rayon::ThreadPoolBuilder::new()
                .num_threads(gen_threads)
                .build()
                .unwrap()
                .install(|| {
                    (0..gpu_batch_size)
                        .into_par_iter()
                        .map(|_| self.chain.generate_address(self.address_type))
                        .unzip()
                });

            // Run on GPU (should be much faster than CPU generation)
            let match_indices = gpu_engine.pattern_match_batch(
                &address_strings,
                &pattern,
                match_type,
                pat_obj.case_insensitive,
            );
            
            // Process matches
            for idx in match_indices {
                if idx >= address_strings.len() {
                    continue;
                }
                
                let address_str = &address_strings[idx];
                let private_key = &keys[idx];
                
                // Double verification (CPU side)
                if self.matcher.matches(address_str).is_some() {
                    // Reconstruct full details for the result
                    if let Some(r) = self.chain.generate_from_bytes(private_key, self.address_type) {
                        stats.mark_found();
                        
                        let total = stats.total_keys();
                        let elapsed = start_time.elapsed().as_secs_f64();
                        
                        result = Some(SearchResult {
                            address: r,
                            pattern: pattern.clone(),
                            keys_tested: total,
                            time_secs: elapsed,
                            keys_per_second: total as f64 / elapsed,
                        });
                        break;
                    }
                }
            }
            
            stats.add_keys(gpu_batch_size as u64);
            
            if result.is_some() {
                break;
            }
        }
        
        stats.stop();
        let _ = printer_handle.join();
        
        result
    }

    /// TURBO MODE: Full GPU key generation for Ed25519 chains (8+ MH/s)
    #[cfg(feature = "opencl")]
    fn run_opencl_turbo(&self) -> Option<SearchResult> {
        let stats = SearchStats::new();
        let start_time = std::time::Instant::now();
        
        // Get pattern info
        let pattern = self.matcher.patterns()
            .first()
            .map(|p| p.value.clone())
            .unwrap_or_default();
        
        let pat_obj = self.matcher.patterns().first().unwrap();
        let case_sensitive = !pat_obj.case_insensitive;
        
        // Determine prefix/suffix from pattern type
        let (prefix, suffix) = match pat_obj.pattern_type {
            PatternType::Prefix => (pattern.as_str(), ""),
            PatternType::Suffix => ("", pattern.as_str()),
            PatternType::Contains => (pattern.as_str(), ""), // Treat as prefix for now
        };
        
        // Initialize OpenCL engine
        let opencl_engine = match OpenClEngine::new(0) {
            Ok(engine) => {
                let est_speed = engine.estimated_keys_per_second();
                info!("🚀 OpenCL TURBO initialized: {} (est. {:.1} MH/s)", 
                    engine.device_info().name,
                    est_speed as f64 / 1_000_000.0
                );
                engine
            }
            Err(e) => {
                info!("OpenCL init failed ({}), falling back to hybrid", e);
                #[cfg(feature = "gpu")]
                {
                    return self.run_gpu_hybrid();
                }
                #[cfg(not(feature = "gpu"))]
                {
                    return self.run_cpu();
                }
            }
        };
        
        // Configure OpenCL search
        let config = OpenClSearchConfig::default();
        let max_time = self.config.max_time_secs;
        let max_attempts = self.config.max_attempts;
        
        // Stats printer thread  
        let stats_for_printer = stats.clone();
        let difficulty = self.difficulty;
        let printer_handle = thread::spawn(move || {
            while stats_for_printer.is_running() {
                eprint!("\r{} 🚀TURBO", stats_for_printer.format(difficulty));
                thread::sleep(Duration::from_millis(250));
            }
            eprintln!();
        });
        
        let mut result: Option<SearchResult> = None;
        let keys_per_iteration = config.global_work_size as u64;
        
        while stats.is_running() {
            // Check limits
            if max_attempts > 0 && stats.total_keys() >= max_attempts {
                break;
            }
            if max_time > 0 && stats.elapsed().as_secs() >= max_time {
                break;
            }
            
            // Run full GPU search iteration
            match opencl_engine.search_ed25519(prefix, suffix, case_sensitive, &config) {
                Ok(Some(private_key)) => {
                    // Found a match! Generate full address details
                    if let Some(addr) = self.chain.generate_from_bytes(&private_key, self.address_type) {
                        // Verify match on CPU (sanity check)
                        if self.matcher.matches(&addr.address).is_some() {
                            stats.mark_found();
                            let total = stats.total_keys();
                            let elapsed = start_time.elapsed().as_secs_f64();
                            
                            result = Some(SearchResult {
                                address: addr,
                                pattern: pattern.clone(),
                                keys_tested: total,
                                time_secs: elapsed,
                                keys_per_second: total as f64 / elapsed,
                            });
                            break;
                        }
                    }
                }
                Ok(None) => {
                    // No match this iteration, continue
                }
                Err(e) => {
                    info!("OpenCL error: {}, stopping search", e);
                    break;
                }
            }
            
            stats.add_keys(keys_per_iteration);
        }
        
        stats.stop();
        let _ = printer_handle.join();
        
        result
    }

    /// Run search with a callback for progress
    pub fn run_with_callback<F>(&self, mut callback: F) -> Option<SearchResult>
    where
        F: FnMut(&SearchStats) + Send,
    {
        let stats = SearchStats::new();
        let stats_clone = stats.clone();

        // Channel for results
        let (tx, rx): (Sender<GeneratedAddress>, Receiver<GeneratedAddress>) = bounded(1);

        // Configure thread pool
        let num_threads = if self.config.threads == 0 {
            num_cpus::get()
        } else {
            self.config.threads
        };

        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build()
            .expect("Failed to create thread pool");

        // Spawn search in background
        let batch_size = self.config.batch_size;
        let chain = self.chain.ticker().to_string();
        let address_type = self.address_type;
        let matcher = self.matcher.clone();
        let stats_for_search = stats.clone();
        let max_attempts = self.config.max_attempts;
        let max_time = self.config.max_time_secs;

        let search_handle = thread::spawn(move || {
            pool.install(|| {
                (0..num_threads).into_par_iter().for_each(|_| {
                    let chain = omnivanity_chains::get_chain(&chain).unwrap();
                    
                    while stats_for_search.is_running() {
                        // Check limits
                        if max_attempts > 0 && stats_for_search.total_keys() >= max_attempts {
                            stats_for_search.stop();
                            break;
                        }
                        if max_time > 0 && stats_for_search.elapsed().as_secs() >= max_time {
                            stats_for_search.stop();
                            break;
                        }

                        for _ in 0..batch_size {
                            let addr = chain.generate(address_type);
                            
                            if matcher.matches(&addr.address).is_some() {
                                let _ = tx.try_send(addr);
                                stats_for_search.mark_found();
                                return;
                            }
                        }

                        stats_for_search.add_keys(batch_size as u64);
                    }
                });
            });
        });

        // Progress callback loop
        while stats.is_running() {
            callback(&stats);
            thread::sleep(Duration::from_millis(100));
        }

        // Wait for search to complete
        let _ = search_handle.join();

        // Return result
        if let Ok(address) = rx.try_recv() {
            let pattern = self.matcher.patterns()
                .first()
                .map(|p| p.value.clone())
                .unwrap_or_default();

            Some(SearchResult {
                address,
                pattern,
                keys_tested: stats.total_keys(),
                time_secs: stats.elapsed().as_secs_f64(),
                keys_per_second: stats.keys_per_second(),
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use omnivanity_chains::ETH;

    #[test]
    fn test_search_easy_pattern() {
        let chain = Box::new(ETH);
        let patterns = vec![Pattern::prefix("0")]; // Very easy pattern
        let config = SearchConfig {
            max_attempts: 100000,
            ..Default::default()
        };

        let search = VanitySearch::new(chain, AddressType::Evm, patterns, config);
        let result = search.run();

        // Should find something starting with 0 quickly
        assert!(result.is_some());
        let result = result.unwrap();
        assert!(result.address.address.to_lowercase().starts_with("0x0"));
    }
}
