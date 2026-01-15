//! GPU search abstraction

use serde::{Deserialize, Serialize};
use omnivanity_chains::{AddressType, GeneratedAddress};

/// GPU search configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuSearchConfig {
    /// Device indices to use (empty = all)
    pub device_indices: Vec<usize>,
    /// Grid size per device (0 = auto)
    pub grid_size: usize,
    /// Block size (threads per block, typically 256/512)
    pub block_size: usize,
    /// Keys per thread per iteration
    pub keys_per_thread: usize,
    /// Maximum attempts (0 = unlimited)
    pub max_attempts: u64,
    /// Maximum time in seconds (0 = unlimited)
    pub max_time_secs: u64,
}

impl Default for GpuSearchConfig {
    fn default() -> Self {
        Self {
            device_indices: vec![],
            grid_size: 0,
            block_size: 256,
            keys_per_thread: 256,
            max_attempts: 0,
            max_time_secs: 0,
        }
    }
}

/// GPU search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuSearchResult {
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
    /// Device that found the match
    pub found_on_device: usize,
}

/// GPU vanity search engine trait
pub trait GpuVanitySearch: Send + Sync {
    /// Chain ticker this engine supports
    fn chain(&self) -> &'static str;
    
    /// Supported address types
    fn address_types(&self) -> Vec<AddressType>;
    
    /// Run GPU search
    fn search(
        &self,
        pattern: &str,
        address_type: AddressType,
        config: &GpuSearchConfig,
    ) -> Option<GpuSearchResult>;
    
    /// Benchmark GPU performance
    fn benchmark(&self, duration_secs: u64, config: &GpuSearchConfig) -> f64;
}
