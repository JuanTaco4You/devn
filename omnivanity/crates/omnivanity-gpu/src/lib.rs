//! OmniVanity GPU Acceleration
//!
//! Cross-platform GPU acceleration using wgpu and OpenCL.

mod device;
mod search;

#[cfg(feature = "wgpu-backend")]
pub mod wgpu_backend;

#[cfg(feature = "opencl-backend")]
pub mod opencl_backend;

pub use device::{GpuDevice, GpuInfo, GpuBackend};
pub use search::{GpuVanitySearch, GpuSearchConfig, GpuSearchResult};

#[cfg(feature = "wgpu-backend")]
pub use wgpu_backend::{WgpuEngine, WgpuError, MatchType, list_wgpu_devices, is_wgpu_available};

#[cfg(feature = "opencl-backend")]
pub use opencl_backend::{OpenClEngine, OpenClError, OpenClDeviceInfo, OpenClSearchConfig, is_opencl_available, list_opencl_devices};

/// Check if GPU acceleration is available
pub fn is_gpu_available() -> bool {
    #[cfg(feature = "wgpu-backend")]
    {
        if wgpu_backend::is_wgpu_available() {
            return true;
        }
    }
    #[cfg(feature = "opencl-backend")]
    {
        if opencl_backend::is_opencl_available() {
            return true;
        }
    }
    false
}

/// Check if OpenCL Turbo mode is available (for full GPU key generation)
pub fn is_turbo_available() -> bool {
    #[cfg(feature = "opencl-backend")]
    {
        return opencl_backend::is_opencl_available();
    }
    #[cfg(not(feature = "opencl-backend"))]
    {
        return false;
    }
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
    #[cfg(feature = "opencl-backend")]
    {
        if opencl_backend::is_opencl_available() {
            return "opencl-turbo";
        }
    }
    #[cfg(feature = "wgpu-backend")]
    {
        return "wgpu";
    }
    #[cfg(not(any(feature = "wgpu-backend", feature = "opencl-backend")))]
    {
        return "none";
    }
}
