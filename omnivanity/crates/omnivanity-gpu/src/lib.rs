//! OmniVanity GPU Acceleration
//!
//! Cross-platform GPU acceleration using wgpu.

mod device;
mod search;

#[cfg(feature = "wgpu-backend")]
pub mod wgpu_backend;

pub use device::{GpuDevice, GpuInfo, GpuBackend};
pub use search::{GpuVanitySearch, GpuSearchConfig, GpuSearchResult};

#[cfg(feature = "wgpu-backend")]
pub use wgpu_backend::{WgpuEngine, WgpuError, list_wgpu_devices, is_wgpu_available};

/// Check if GPU acceleration is available
pub fn is_gpu_available() -> bool {
    #[cfg(feature = "wgpu-backend")]
    {
        if wgpu_backend::is_wgpu_available() {
            return true;
        }
    }
    false
}

/// Get list of available GPU devices
pub fn list_devices() -> Vec<GpuDevice> {
    let mut devices = vec![];
    
    #[cfg(feature = "wgpu-backend")]
    {
        devices.extend(wgpu_backend::list_wgpu_devices());
    }
    
    devices
}

/// Get the preferred GPU backend name
pub fn preferred_backend() -> &'static str {
    #[cfg(feature = "wgpu-backend")]
    {
        return "wgpu";
    }
    #[cfg(not(feature = "wgpu-backend"))]
    {
        return "none";
    }
}
