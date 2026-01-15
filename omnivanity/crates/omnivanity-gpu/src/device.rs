//! GPU device abstraction

use serde::{Deserialize, Serialize};

/// GPU device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuDevice {
    /// Device index
    pub index: usize,
    /// Device name
    pub name: String,
    /// Compute capability (e.g., "8.6" for Ampere)
    pub compute_capability: String,
    /// Total memory in bytes
    pub total_memory: u64,
    /// Number of multiprocessors/compute units
    pub multiprocessors: u32,
    /// Backend type (CUDA, OpenCL, etc.)
    pub backend: GpuBackend,
}

/// GPU backend type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GpuBackend {
    Cuda,
    OpenCL,
    Vulkan,
    Metal,
    Dx12,
    Wgpu,
}

impl std::fmt::Display for GpuBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GpuBackend::Cuda => write!(f, "CUDA"),
            GpuBackend::OpenCL => write!(f, "OpenCL"),
            GpuBackend::Vulkan => write!(f, "Vulkan"),
            GpuBackend::Metal => write!(f, "Metal"),
            GpuBackend::Dx12 => write!(f, "DirectX 12"),
            GpuBackend::Wgpu => write!(f, "wgpu"),
        }
    }
}

impl GpuDevice {
    /// Format memory size
    pub fn memory_formatted(&self) -> String {
        let gb = self.total_memory as f64 / (1024.0 * 1024.0 * 1024.0);
        format!("{:.1} GB", gb)
    }
}

/// Summary info for all GPUs
#[derive(Debug, Clone, Default)]
pub struct GpuInfo {
    pub devices: Vec<GpuDevice>,
    pub total_memory: u64,
    pub total_multiprocessors: u32,
}

impl GpuInfo {
    pub fn new(devices: Vec<GpuDevice>) -> Self {
        let total_memory = devices.iter().map(|d| d.total_memory).sum();
        let total_multiprocessors = devices.iter().map(|d| d.multiprocessors).sum();
        Self {
            devices,
            total_memory,
            total_multiprocessors,
        }
    }
}
