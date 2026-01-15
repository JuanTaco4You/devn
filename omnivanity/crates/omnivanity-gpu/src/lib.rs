//! OmniVanity GPU Acceleration
//!
//! CUDA kernels for high-speed vanity address generation.

mod device;
mod search;

#[cfg(feature = "cuda")]
mod cuda;

pub use device::{GpuDevice, GpuInfo};
pub use search::{GpuVanitySearch, GpuSearchConfig, GpuSearchResult};

/// Check if GPU acceleration is available
pub fn is_gpu_available() -> bool {
    #[cfg(feature = "cuda")]
    {
        cuda::is_cuda_available()
    }
    #[cfg(not(feature = "cuda"))]
    {
        false
    }
}

/// Get list of available GPU devices
pub fn list_devices() -> Vec<GpuDevice> {
    #[cfg(feature = "cuda")]
    {
        cuda::list_cuda_devices()
    }
    #[cfg(not(feature = "cuda"))]
    {
        vec![]
    }
}
