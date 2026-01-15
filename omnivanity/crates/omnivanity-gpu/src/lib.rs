//! OmniVanity GPU Acceleration
//!
//! Cross-platform GPU acceleration using wgpu (default) with optional CUDA backend.

mod device;
mod search;

// wgpu backend (Tier 1 - default)
#[cfg(feature = "wgpu-backend")]
pub mod wgpu_backend;

// CUDA backend (Tier 2 - optional, NVIDIA only)
#[cfg(feature = "cuda")]
pub mod cuda;

#[cfg(feature = "cuda")]
pub mod evm_engine;

pub use device::{GpuDevice, GpuInfo, GpuBackend};
pub use search::{GpuVanitySearch, GpuSearchConfig, GpuSearchResult};

#[cfg(feature = "wgpu-backend")]
pub use wgpu_backend::{WgpuEngine, WgpuError, list_wgpu_devices, is_wgpu_available};

#[cfg(feature = "cuda")]
pub use cuda::{is_cuda_available, list_cuda_devices, get_cuda_info, CudaError};

#[cfg(feature = "cuda")]
pub use evm_engine::{EvmCudaEngine, EvmCudaError};

/// Check if GPU acceleration is available (any backend)
pub fn is_gpu_available() -> bool {
    #[cfg(feature = "wgpu-backend")]
    {
        if wgpu_backend::is_wgpu_available() {
            return true;
        }
    }
    #[cfg(feature = "cuda")]
    {
        if cuda::is_cuda_available() {
            return true;
        }
    }
    false
}

/// Get list of available GPU devices (from all backends)
pub fn list_devices() -> Vec<GpuDevice> {
    let mut devices = vec![];
    
    #[cfg(feature = "wgpu-backend")]
    {
        devices.extend(wgpu_backend::list_wgpu_devices());
    }
    
    // Note: Don't duplicate if wgpu already found the CUDA devices
    #[cfg(all(feature = "cuda", not(feature = "wgpu-backend")))]
    {
        devices.extend(cuda::list_cuda_devices());
    }
    
    devices
}

/// Get the preferred GPU backend name
pub fn preferred_backend() -> &'static str {
    #[cfg(feature = "wgpu-backend")]
    {
        return "wgpu";
    }
    #[cfg(all(feature = "cuda", not(feature = "wgpu-backend")))]
    {
        return "cuda";
    }
    #[cfg(not(any(feature = "wgpu-backend", feature = "cuda")))]
    {
        return "none";
    }
}
