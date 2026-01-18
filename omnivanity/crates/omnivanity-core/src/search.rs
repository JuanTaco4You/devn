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
        // Check if GPU should be used
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
        let gpu = match WgpuEngine::new_sync(0, gpu_config) {
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
        let pattern_type = self.matcher.patterns()
            .first()
            .map(|p| p.pattern_type)
            .unwrap_or(PatternType::Prefix);
        let case_insensitive = self.matcher.patterns()
            .first()
            .map(|p| p.case_insensitive)
            .unwrap_or(false);
        
        let match_type = match pattern_type {
            PatternType::Prefix => MatchType::Prefix,
            PatternType::Suffix => MatchType::Suffix,
            PatternType::Contains => MatchType::Contains,
        };
        
        // Batch size for GPU (larger = more GPU utilization)
        let gpu_batch_size = 8192;
        let max_time = self.config.max_time_secs;
        let max_attempts = self.config.max_attempts;
        
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
            
            // Generate batch of addresses on CPU (using rayon)
            let batch: Vec<GeneratedAddress> = (0..gpu_batch_size)
                .into_par_iter()
                .map(|_| self.chain.generate(self.address_type))
                .collect();
            
            // Extract address strings for GPU matching
            let address_strings: Vec<String> = batch.iter()
                .map(|a| {
                    // Strip prefix for matching (e.g., "0x" for EVM)
                    let prefix = self.chain.address_prefix(self.address_type);
                    if a.address.starts_with(prefix) && !prefix.is_empty() {
                        a.address[prefix.len()..].to_string()
                    } else {
                        a.address.clone()
                    }
                })
                .collect();
            
            // GPU pattern matching
            let matches = gpu.pattern_match_batch(
                &address_strings,
                &pattern,
                match_type,
                case_insensitive,
            );
            
            stats.add_keys(gpu_batch_size as u64);
            
            // Check for matches and verify on CPU
            for match_idx in matches {
                if let Some(matched_address) = batch.get(match_idx) {
                    // Double check with CPU matcher to ensure correctness (handles prefixes, etc.)
                    if self.matcher.matches(&matched_address.address).is_some() {
                        stats.mark_found();
                        result = Some(SearchResult {
                            address: matched_address.clone(),
                            pattern: pattern.clone(),
                            keys_tested: stats.total_keys(),
                            time_secs: stats.elapsed().as_secs_f64(),
                            keys_per_second: stats.keys_per_second(),
                        });
                        break;
                    }
                }
            }
            
            if result.is_some() {
                break;
            }
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
