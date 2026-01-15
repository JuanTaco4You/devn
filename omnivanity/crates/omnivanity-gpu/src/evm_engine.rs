//! EVM CUDA Engine
//!
//! GPU-accelerated vanity address generation for EVM chains (ETH, etc.)

use crate::device::{GpuBackend, GpuDevice};
use crate::search::{GpuSearchConfig, GpuSearchResult, GpuVanitySearch};
use omnivanity_chains::{AddressType, GeneratedAddress};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

#[cfg(feature = "cuda")]
use cudarc::driver::{CudaDevice, CudaSlice, LaunchAsync, LaunchConfig};
#[cfg(feature = "cuda")]
use cudarc::nvrtc::compile_ptx;

/// CUDA kernel source for EVM keccak256 vanity generation
const EVM_KERNEL_SRC: &str = include_str!("kernels/evm_kernel.cu");

/// EVM CUDA Engine for GPU vanity search
pub struct EvmCudaEngine {
    #[cfg(feature = "cuda")]
    device: Arc<CudaDevice>,
    device_index: usize,
    config: GpuSearchConfig,
}

impl EvmCudaEngine {
    /// Create a new EVM CUDA engine
    #[cfg(feature = "cuda")]
    pub fn new(device_index: usize, config: GpuSearchConfig) -> Result<Self, EvmCudaError> {
        let device = CudaDevice::new(device_index)?;
        
        info!(
            "Initialized EVM CUDA engine on device {}: {}",
            device_index,
            device.name().unwrap_or_default()
        );
        
        Ok(Self {
            device: Arc::new(device),
            device_index,
            config,
        })
    }
    
    #[cfg(not(feature = "cuda"))]
    pub fn new(device_index: usize, config: GpuSearchConfig) -> Result<Self, EvmCudaError> {
        Err(EvmCudaError::NotEnabled)
    }

    /// Search for a vanity EVM address
    #[cfg(feature = "cuda")]
    pub fn search(
        &self,
        pattern: &[u8],
        pattern_len: usize,
        case_insensitive: bool,
        stop_flag: Arc<AtomicBool>,
    ) -> Option<GpuSearchResult> {
        // Configuration
        let block_size = self.config.block_size as u32;
        let grid_size = if self.config.grid_size == 0 {
            (self.device.num_sms() * 4) as u32
        } else {
            self.config.grid_size as u32
        };
        let keys_per_thread = self.config.keys_per_thread;
        let total_threads = (grid_size * block_size) as usize;
        
        info!(
            "Launching EVM search: {} blocks x {} threads x {} keys/thread = {} keys/iteration",
            grid_size,
            block_size,
            keys_per_thread,
            total_threads * keys_per_thread
        );

        // Compile kernel
        let ptx = match compile_ptx(EVM_KERNEL_SRC) {
            Ok(p) => p,
            Err(e) => {
                warn!("Failed to compile EVM kernel: {:?}", e);
                return None;
            }
        };
        
        if let Err(e) = self.device.load_ptx(ptx, "evm_search", &["evm_vanity_search", "evm_benchmark"]) {
            warn!("Failed to load PTX: {:?}", e);
            return None;
        }
        
        let func = match self.device.get_func("evm_search", "evm_vanity_search") {
            Some(f) => f,
            None => {
                warn!("Kernel function not found");
                return None;
            }
        };

        // Allocate buffers
        let seeds_host: Vec<u64> = (0..total_threads * 4)
            .map(|i| {
                // Mix device index and thread index into seed
                let base = rand::random::<u64>();
                base ^ (i as u64) ^ ((self.device_index as u64) << 48)
            })
            .collect();
        
        let seeds_dev = match self.device.htod_sync_copy(&seeds_host) {
            Ok(s) => s,
            Err(e) => {
                warn!("Failed to copy seeds to device: {:?}", e);
                return None;
            }
        };
        
        // Output buffers
        let found_flags = self.device.alloc_zeros::<u8>(total_threads).ok()?;
        let found_privkeys = self.device.alloc_zeros::<u8>(total_threads * 32).ok()?;
        let found_addresses = self.device.alloc_zeros::<u8>(total_threads * 20).ok()?;
        
        // Pattern buffer
        let pattern_dev = self.device.htod_sync_copy(pattern).ok()?;
        
        let cfg = LaunchConfig {
            block_dim: (block_size, 1, 1),
            grid_dim: (grid_size, 1, 1),
            shared_mem_bytes: 0,
        };

        let start = Instant::now();
        let max_time = Duration::from_secs(self.config.max_time_secs);
        let mut total_keys = 0u64;
        let mut iteration = 0u32;

        loop {
            // Check stop conditions
            if stop_flag.load(Ordering::Relaxed) {
                break;
            }
            
            if self.config.max_time_secs > 0 && start.elapsed() > max_time {
                break;
            }
            
            if self.config.max_attempts > 0 && total_keys >= self.config.max_attempts {
                break;
            }

            // Launch kernel
            unsafe {
                if let Err(e) = func.launch(
                    cfg.clone(),
                    (
                        &seeds_dev,
                        &found_flags,
                        &found_privkeys,
                        &found_addresses,
                        &pattern_dev,
                        pattern_len as i32,
                        keys_per_thread as i32,
                        iteration as i32,
                    ),
                ) {
                    warn!("Kernel launch failed: {:?}", e);
                    break;
                }
            }
            
            if let Err(e) = self.device.synchronize() {
                warn!("Sync failed: {:?}", e);
                break;
            }

            // Check for results
            let mut flags_host = vec![0u8; total_threads];
            if let Err(e) = self.device.dtoh_sync_copy_into(&found_flags, &mut flags_host) {
                warn!("Failed to copy flags: {:?}", e);
                break;
            }

            // Check if any thread found a match
            for (thread_idx, &found) in flags_host.iter().enumerate() {
                if found != 0 {
                    // Found a match! Copy the result
                    let mut privkey = vec![0u8; 32];
                    let mut address = vec![0u8; 20];
                    
                    // TODO: Copy specific thread's result
                    // For now, we'd need to copy the full buffer and extract
                    
                    let elapsed = start.elapsed().as_secs_f64();
                    let keys_per_second = total_keys as f64 / elapsed;
                    
                    info!(
                        "Match found on device {} thread {} after {} keys",
                        self.device_index,
                        thread_idx,
                        total_keys
                    );
                    
                    return Some(GpuSearchResult {
                        address: GeneratedAddress {
                            address: format!("0x{}", hex::encode(&address)),
                            private_key_hex: hex::encode(&privkey),
                            private_key_native: hex::encode(&privkey),
                            public_key_hex: String::new(),
                        },
                        pattern: String::new(),
                        keys_tested: total_keys,
                        time_secs: elapsed,
                        keys_per_second,
                        found_on_device: self.device_index,
                    });
                }
            }

            total_keys += (total_threads * keys_per_thread) as u64;
            iteration += 1;
            
            // Log progress every 10 iterations
            if iteration % 10 == 0 {
                let elapsed = start.elapsed().as_secs_f64();
                let rate = total_keys as f64 / elapsed / 1_000_000.0;
                debug!(
                    "GPU {}: {} keys tested ({:.2} Mkey/s)",
                    self.device_index,
                    total_keys,
                    rate
                );
            }
        }

        None
    }
    
    #[cfg(not(feature = "cuda"))]
    pub fn search(
        &self,
        _pattern: &[u8],
        _pattern_len: usize,
        _case_insensitive: bool,
        _stop_flag: Arc<AtomicBool>,
    ) -> Option<GpuSearchResult> {
        None
    }

    /// Benchmark GPU keccak throughput
    #[cfg(feature = "cuda")]
    pub fn benchmark(&self, duration_secs: u64) -> Result<f64, EvmCudaError> {
        let block_size = self.config.block_size as u32;
        let grid_size = if self.config.grid_size == 0 {
            (self.device.num_sms() * 4) as u32
        } else {
            self.config.grid_size as u32
        };
        let keys_per_thread = self.config.keys_per_thread;
        let total_threads = (grid_size * block_size) as usize;
        
        // Compile kernel
        let ptx = compile_ptx(EVM_KERNEL_SRC)?;
        self.device.load_ptx(ptx, "evm_bench", &["evm_benchmark"])?;
        
        let func = self.device.get_func("evm_bench", "evm_benchmark")
            .ok_or(EvmCudaError::KernelNotFound)?;
        
        // Allocate buffers
        let seeds_host: Vec<u64> = (0..total_threads * 4).map(|i| i as u64).collect();
        let seeds_dev = self.device.htod_sync_copy(&seeds_host)?;
        let counter_dev = self.device.alloc_zeros::<u64>(1)?;
        
        let cfg = LaunchConfig {
            block_dim: (block_size, 1, 1),
            grid_dim: (grid_size, 1, 1),
            shared_mem_bytes: 0,
        };
        
        // Warmup
        unsafe {
            func.launch(cfg.clone(), (&seeds_dev, &counter_dev, keys_per_thread as i32))?;
        }
        self.device.synchronize()?;
        
        // Timed runs
        let start = Instant::now();
        let mut total_keys = 0u64;
        let max_time = Duration::from_secs(duration_secs);
        
        while start.elapsed() < max_time {
            unsafe {
                func.launch(cfg.clone(), (&seeds_dev, &counter_dev, keys_per_thread as i32))?;
            }
            self.device.synchronize()?;
            total_keys += (total_threads * keys_per_thread) as u64;
        }
        
        let elapsed = start.elapsed().as_secs_f64();
        let keys_per_second = total_keys as f64 / elapsed;
        
        Ok(keys_per_second)
    }
    
    #[cfg(not(feature = "cuda"))]
    pub fn benchmark(&self, _duration_secs: u64) -> Result<f64, EvmCudaError> {
        Err(EvmCudaError::NotEnabled)
    }
}

impl GpuVanitySearch for EvmCudaEngine {
    fn chain(&self) -> &'static str {
        "ETH"
    }
    
    fn address_types(&self) -> Vec<AddressType> {
        vec![AddressType::Evm]
    }
    
    fn search(
        &self,
        pattern: &str,
        address_type: AddressType,
        config: &GpuSearchConfig,
    ) -> Option<GpuSearchResult> {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let pattern_bytes = hex::decode(pattern.trim_start_matches("0x")).unwrap_or_default();
        self.search(&pattern_bytes, pattern.len(), false, stop_flag)
    }
    
    fn benchmark(&self, duration_secs: u64, _config: &GpuSearchConfig) -> f64 {
        self.benchmark(duration_secs).unwrap_or(0.0)
    }
}

/// EVM CUDA error type
#[derive(Debug, thiserror::Error)]
pub enum EvmCudaError {
    #[cfg(feature = "cuda")]
    #[error("CUDA driver error: {0}")]
    DriverError(#[from] cudarc::driver::DriverError),
    #[cfg(feature = "cuda")]
    #[error("NVRTC compilation error: {0}")]
    CompileError(#[from] cudarc::nvrtc::CompileError),
    #[error("No CUDA devices found")]
    NoDevices,
    #[error("Kernel not found")]
    KernelNotFound,
    #[error("CUDA not enabled")]
    NotEnabled,
}
