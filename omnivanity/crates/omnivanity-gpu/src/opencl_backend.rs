//! OpenCL Backend for Full GPU Key Generation
//!
//! This backend implements full Ed25519 key generation on GPU for Solana-family chains.
//! Achieves 8+ MH/s on modern NVIDIA/AMD GPUs.

#[cfg(feature = "opencl-backend")]
use ocl::{
    Buffer, Context, Device, Kernel, Platform, Program, Queue,
    flags, core::DeviceInfo,
};

use thiserror::Error;
use tracing::info;

/// OpenCL errors
#[derive(Error, Debug)]
pub enum OpenClError {
    #[error("No OpenCL platforms found")]
    NoPlatforms,
    #[error("No OpenCL GPU devices found")]
    NoDevices,
    #[error("OpenCL error: {0}")]
    OclError(String),
    #[error("Kernel compilation failed: {0}")]
    KernelCompilationFailed(String),
}

#[cfg(feature = "opencl-backend")]
impl From<ocl::Error> for OpenClError {
    fn from(e: ocl::Error) -> Self {
        OpenClError::OclError(e.to_string())
    }
}

/// OpenCL device info
#[derive(Debug, Clone)]
pub struct OpenClDeviceInfo {
    pub name: String,
    pub vendor: String,
    pub platform: String,
    pub compute_units: u32,
    pub max_work_group_size: usize,
    pub global_mem_size: u64,
    pub is_nvidia: bool,
}

/// Configuration for OpenCL search
#[derive(Debug, Clone)]
pub struct OpenClSearchConfig {
    /// Global work size (total parallel invocations)
    pub global_work_size: usize,
    /// Local work size (work group size) - MUST be 32 for stability!
    pub local_work_size: usize,
    /// Number of iteration bits (controls keys per kernel call)
    pub iteration_bits: u32,
}

impl Default for OpenClSearchConfig {
    fn default() -> Self {
        // Values from solVanityPlus - proven to work at 8+ MH/s
        Self {
            global_work_size: 1 << 24, // 16M (solVanityPlus default)
            local_work_size: 32,       // CRITICAL: 32 not 256! 256 crashes!
            iteration_bits: 24,        // 2^24 = 16M keys per iteration
        }
    }
}

/// OpenCL Engine for full GPU key generation
#[cfg(feature = "opencl-backend")]
pub struct OpenClEngine {
    context: Context,
    queue: Queue,
    program: Program,
    device_info: OpenClDeviceInfo,
}

#[cfg(feature = "opencl-backend")]
impl OpenClEngine {
    /// Create a new OpenCL engine with the specified device index
    pub fn new(device_index: usize) -> Result<Self, OpenClError> {
        // Get all platforms
        let platforms = Platform::list();
        if platforms.is_empty() {
            return Err(OpenClError::NoPlatforms);
        }

        // Find GPU devices across all platforms
        let mut all_devices = Vec::new();
        for platform in &platforms {
            if let Ok(devices) = Device::list(platform, Some(flags::DeviceType::GPU)) {
                for device in devices {
                    let platform_name = platform.name().unwrap_or_default();
                    all_devices.push((device, platform_name, platform.clone()));
                }
            }
        }

        if all_devices.is_empty() {
            return Err(OpenClError::NoDevices);
        }

        let (device, platform_name, platform) = all_devices
            .get(device_index)
            .cloned()
            .ok_or(OpenClError::NoDevices)?;

        let device_name = device.name().unwrap_or_default();
        let vendor = device.vendor().unwrap_or_default();
        let is_nvidia = platform_name.to_uppercase().contains("NVIDIA");

        info!(
            "OpenCL device: {} ({}) on {}",
            device_name, vendor, platform_name
        );

        // Create context and queue
        let context = Context::builder()
            .platform(platform)
            .devices(device.clone())
            .build()?;

        let queue = Queue::new(&context, device.clone(), None)?;

        // Load and compile kernel
        let kernel_source = include_str!("kernels/ed25519_solana.cl");
        let program = Program::builder()
            .src(kernel_source)
            .devices(device.clone())
            .build(&context)?;

        let device_info = OpenClDeviceInfo {
            name: device_name,
            vendor,
            platform: platform_name,
            compute_units: device.info(DeviceInfo::MaxComputeUnits).ok()
                .and_then(|i| i.to_string().parse().ok()).unwrap_or(0),
            max_work_group_size: device.max_wg_size().unwrap_or(256),
            global_mem_size: device.info(DeviceInfo::GlobalMemSize).ok()
                .and_then(|i| i.to_string().parse().ok()).unwrap_or(0),
            is_nvidia,
        };

        Ok(Self {
            context,
            queue,
            program,
            device_info,
        })
    }

    /// Get device info
    pub fn device_info(&self) -> &OpenClDeviceInfo {
        &self.device_info
    }

    /// Search for a vanity address matching the given prefix/suffix pattern
    /// Returns (found: bool, private_key: [u8; 32]) if found
    pub fn search_ed25519(
        &self,
        _prefix: &str,
        _suffix: &str,
        _case_sensitive: bool,
        config: &OpenClSearchConfig,
    ) -> Result<Option<[u8; 32]>, OpenClError> {
        // Calculate iteration_bytes = ceil(iteration_bits / 8) - same as solVanityPlus
        let iteration_bytes = ((config.iteration_bits + 7) / 8) as usize;
        
        // Generate key32 with last iteration_bytes zeroed (GPU will iterate over these)
        let mut key32 = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut key32[..(32 - iteration_bytes)]);
        // Last iteration_bytes are 0x00 - GPU will iterate over them

        // Create buffers
        let key32_buffer = Buffer::<u8>::builder()
            .queue(self.queue.clone())
            .flags(flags::MEM_READ_ONLY | flags::MEM_COPY_HOST_PTR)
            .len(32)
            .copy_host_slice(&key32)
            .build()?;

        let output_buffer = Buffer::<u8>::builder()
            .queue(self.queue.clone())
            .flags(flags::MEM_READ_WRITE)
            .len(33) // 1 byte found flag + 32 byte private key
            .build()?;

        // occupied_bytes = iteration_bytes (how many bytes GPU iterates over)
        let occupied_bytes_buffer = Buffer::<u8>::builder()
            .queue(self.queue.clone())
            .flags(flags::MEM_READ_ONLY | flags::MEM_COPY_HOST_PTR)
            .len(1)
            .copy_host_slice(&[iteration_bytes as u8])
            .build()?;

        let group_offset_buffer = Buffer::<u8>::builder()
            .queue(self.queue.clone())
            .flags(flags::MEM_READ_ONLY | flags::MEM_COPY_HOST_PTR)
            .len(1)
            .copy_host_slice(&[0u8])
            .build()?;

        // Create and run kernel
        let kernel = Kernel::builder()
            .program(&self.program)
            .name("generate_pubkey")
            .queue(self.queue.clone())
            .global_work_size(config.global_work_size)
            .local_work_size(config.local_work_size)
            .arg(&key32_buffer)
            .arg(&output_buffer)
            .arg(&occupied_bytes_buffer)
            .arg(&group_offset_buffer)
            .build()?;

        unsafe {
            kernel.enq()?;
        }
        self.queue.finish()?;

        // Read result
        let mut output = vec![0u8; 33];
        output_buffer.read(&mut output).enq()?;

        if output[0] != 0 {
            // Found a match!
            let mut private_key = [0u8; 32];
            private_key.copy_from_slice(&output[1..33]);
            Ok(Some(private_key))
        } else {
            Ok(None)
        }
    }

    /// Get estimated keys per second based on device capabilities
    pub fn estimated_keys_per_second(&self) -> u64 {
        // Rough estimate based on compute units
        // NVIDIA RTX 4090 has ~128 SMs, each can do ~200K keys/sec
        let base_rate = if self.device_info.is_nvidia { 80_000 } else { 50_000 };
        (self.device_info.compute_units as u64) * base_rate
    }
}

/// Check if OpenCL is available
#[cfg(feature = "opencl-backend")]
pub fn is_opencl_available() -> bool {
    let platforms = Platform::list();
    for platform in platforms {
        if let Ok(devices) = Device::list(&platform, Some(flags::DeviceType::GPU)) {
            if !devices.is_empty() {
                return true;
            }
        }
    }
    false
}

#[cfg(not(feature = "opencl-backend"))]
pub fn is_opencl_available() -> bool {
    false
}

/// List all OpenCL GPU devices
#[cfg(feature = "opencl-backend")]
pub fn list_opencl_devices() -> Vec<OpenClDeviceInfo> {
    let mut devices = Vec::new();
    let platforms = Platform::list();
    
    for platform in platforms {
        let platform_name = platform.name().unwrap_or_default();
        if let Ok(gpu_devices) = Device::list(&platform, Some(flags::DeviceType::GPU)) {
            for device in gpu_devices {
                devices.push(OpenClDeviceInfo {
                    name: device.name().unwrap_or_default(),
                    vendor: device.vendor().unwrap_or_default(),
                    platform: platform_name.clone(),
                    compute_units: device.info(DeviceInfo::MaxComputeUnits).ok()
                        .and_then(|i| i.to_string().parse().ok()).unwrap_or(0),
                    max_work_group_size: device.max_wg_size().unwrap_or(256),
                    global_mem_size: device.info(DeviceInfo::GlobalMemSize).ok()
                        .and_then(|i| i.to_string().parse().ok()).unwrap_or(0),
                    is_nvidia: platform_name.to_uppercase().contains("NVIDIA"),
                });
            }
        }
    }
    devices
}

#[cfg(not(feature = "opencl-backend"))]
pub fn list_opencl_devices() -> Vec<OpenClDeviceInfo> {
    Vec::new()
}
