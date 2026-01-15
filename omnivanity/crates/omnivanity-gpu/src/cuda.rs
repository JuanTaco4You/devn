//! CUDA backend implementation with runtime compilation

use crate::device::{GpuBackend, GpuDevice};
use std::time::Instant;
use tracing::info;

#[cfg(feature = "cuda")]
use cudarc::driver::CudaDevice;

/// Check if CUDA is available
pub fn is_cuda_available() -> bool {
    #[cfg(feature = "cuda")]
    {
        CudaDevice::new(0).is_ok()
    }
    #[cfg(not(feature = "cuda"))]
    {
        false
    }
}

/// List all CUDA devices
pub fn list_cuda_devices() -> Vec<GpuDevice> {
    #[cfg(not(feature = "cuda"))]
    {
        return vec![];
    }
    
    #[cfg(feature = "cuda")]
    {
        let mut devices = vec![];
        
        for i in 0..16 {
            match CudaDevice::new(i) {
                Ok(dev) => {
                    // Use the CudaDevice methods that are available
                    let name = format!("CUDA Device {}", i);
                    
                    devices.push(GpuDevice {
                        index: i,
                        name,
                        compute_capability: "N/A".to_string(),
                        total_memory: 0,
                        multiprocessors: 0,
                        backend: GpuBackend::Cuda,
                    });
                }
                Err(_) => break,
            }
        }
        
        devices
    }
}

/// Get info about all CUDA devices
pub fn get_cuda_info() -> String {
    let devices = list_cuda_devices();
    if devices.is_empty() {
        return "No CUDA devices found. Make sure NVIDIA drivers and CUDA toolkit are installed.".to_string();
    }
    
    let mut info = format!("Found {} CUDA device(s):\n", devices.len());
    for dev in &devices {
        info.push_str(&format!(
            "  [{}] {} - {}\n",
            dev.index,
            dev.name,
            dev.backend
        ));
    }
    info
}

/// CUDA error type
#[derive(Debug, thiserror::Error)]
pub enum CudaError {
    #[cfg(feature = "cuda")]
    #[error("CUDA driver error: {0}")]
    DriverError(#[from] cudarc::driver::DriverError),
    #[error("No CUDA devices found")]
    NoDevices,
    #[error("Invalid device index: {0}")]
    InvalidDevice(usize),
    #[error("Kernel not found")]
    KernelNotFound,
    #[error("CUDA not enabled")]
    NotEnabled,
}

/// CUDA context for EVM vanity generation
#[cfg(feature = "cuda")]
pub struct CudaEvmEngine {
    device: std::sync::Arc<CudaDevice>,
    device_index: usize,
}

#[cfg(feature = "cuda")]
impl CudaEvmEngine {
    /// Create new CUDA context for EVM
    pub fn new(device_index: usize) -> Result<Self, CudaError> {
        let device: std::sync::Arc<CudaDevice> = CudaDevice::new(device_index)?;
        
        info!("Initialized CUDA device {}", device_index);
        
        Ok(Self { device, device_index })
    }

    /// Benchmark GPU throughput (placeholder)
    pub fn benchmark(&self, _duration_secs: u64) -> Result<f64, CudaError> {
        // For now, return a placeholder speed
        // Full implementation would compile and run the kernel
        Ok(0.0)
    }
    
    /// Get device index
    pub fn device_index(&self) -> usize {
        self.device_index
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cuda_available() {
        let available = is_cuda_available();
        println!("CUDA available: {}", available);
    }

    #[test]
    fn test_list_devices() {
        let devices = list_cuda_devices();
        println!("Found {} CUDA devices", devices.len());
        for dev in &devices {
            println!("  [{}] {}", dev.index, dev.name);
        }
    }
}
