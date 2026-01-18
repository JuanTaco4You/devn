//! wgpu GPU Backend
//!
//! Cross-platform GPU acceleration using wgpu (Vulkan/Metal/DX12).
//! This is the default Tier 1 backend for OmniVanity.

#[cfg(feature = "wgpu-backend")]
use wgpu::{
    Adapter, Device, Queue, ShaderModule, ComputePipeline, 
    Buffer, BufferUsages, BindGroup, BindGroupLayout,
    util::DeviceExt,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{info, warn, debug};

use crate::device::{GpuDevice, GpuBackend, GpuInfo};
use crate::search::{GpuSearchConfig, GpuSearchResult};

/// EVM WGSL shader source
const EVM_SHADER: &str = include_str!("kernels/evm.wgsl");

/// Pattern matching WGSL shader source (generic, works with any chain)
const PATTERN_MATCH_SHADER: &str = include_str!("kernels/pattern_match.wgsl");

/// Match type enum for pattern matching
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchType {
    Prefix = 0,
    Suffix = 1,
    Contains = 2,
}

/// wgpu GPU Engine
#[cfg(feature = "wgpu-backend")]
pub struct WgpuEngine {
    device: Device,
    queue: Queue,
    adapter_info: wgpu::AdapterInfo,
    config: GpuSearchConfig,
}

#[cfg(feature = "wgpu-backend")]
impl WgpuEngine {
    /// Create a new wgpu engine for the specified device
    pub async fn new(device_index: usize, config: GpuSearchConfig) -> Result<Self, WgpuError> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        
        let adapters: Vec<wgpu::Adapter> = instance.enumerate_adapters(wgpu::Backends::all());
        if adapters.is_empty() {
            return Err(WgpuError::NoDevices);
        }
        
        let adapter = adapters.into_iter()
            .nth(device_index)
            .ok_or(WgpuError::DeviceNotFound(device_index))?;
        
        let adapter_info = adapter.get_info();
        info!(
            "Using GPU: {} ({:?})",
            adapter_info.name,
            adapter_info.backend
        );
        
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("OmniVanity GPU"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await
            .map_err(|e: wgpu::RequestDeviceError| WgpuError::DeviceRequest(e.to_string()))?;
        
        Ok(Self {
            device,
            queue,
            adapter_info,
            config,
        })
    }
    
    /// Create synchronously using pollster
    pub fn new_sync(device_index: usize, config: GpuSearchConfig) -> Result<Self, WgpuError> {
        pollster::block_on(Self::new(device_index, config))
    }
    
    /// Get device name
    pub fn device_name(&self) -> &str {
        &self.adapter_info.name
    }
    
    /// Run EVM vanity search
    pub fn search_evm(
        &self,
        pattern: &[u8],
        pattern_len: usize,
        stop_flag: Arc<AtomicBool>,
    ) -> Option<GpuSearchResult> {
        let workgroup_size = 256u32;
        let num_workgroups = if self.config.grid_size == 0 {
            256u32  // Auto: 256 workgroups * 256 threads = 65536 threads
        } else {
            self.config.grid_size as u32
        };
        let total_threads = (workgroup_size * num_workgroups) as usize;
        let keys_per_thread = self.config.keys_per_thread;
        
        info!(
            "Launching EVM search: {} workgroups x {} threads x {} keys/thread",
            num_workgroups,
            workgroup_size,
            keys_per_thread
        );
        
        // Compile shader
        let shader = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("EVM Vanity Shader"),
            source: wgpu::ShaderSource::Wgsl(EVM_SHADER.into()),
        });
        
        // Create buffers
        let seeds: Vec<[u32; 4]> = (0..total_threads)
            .map(|i| {
                let base = rand::random::<u64>();
                [
                    (base & 0xFFFFFFFF) as u32,
                    ((base >> 32) & 0xFFFFFFFF) as u32,
                    (i as u32) ^ 0x12345678,
                    rand::random::<u32>(),
                ]
            })
            .collect();
        
        let seeds_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Seeds Buffer"),
            contents: bytemuck::cast_slice(&seeds),
            usage: BufferUsages::STORAGE,
        });
        
        // Pattern buffer (pad to at least 16 bytes)
        let mut pattern_data = vec![0u32; 4];
        for (i, &b) in pattern.iter().enumerate() {
            let word_idx = i / 4;
            let shift = (i % 4) * 8;
            if word_idx < pattern_data.len() {
                pattern_data[word_idx] |= (b as u32) << shift;
            }
        }
        
        let pattern_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Pattern Buffer"),
            contents: bytemuck::cast_slice(&pattern_data),
            usage: BufferUsages::STORAGE,
        });
        
        // Params uniform
        #[repr(C)]
        #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
        struct SearchParams {
            pattern_len: u32,
            iteration: u32,
            keys_per_thread: u32,
            _padding: u32,
        }
        
        // Results buffer
        #[repr(C)]
        #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
        struct SearchResult {
            found: u32,
            thread_id: u32,
            _padding1: u32,
            _padding2: u32,
        }
        
        let results_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Results Buffer"),
            size: (total_threads * std::mem::size_of::<SearchResult>()) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        
        let found_keys_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Found Keys Buffer"),
            size: (total_threads * 32) as u64,  // 32 bytes per key
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        
        let found_addrs_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Found Addresses Buffer"),
            size: (total_threads * 20) as u64,  // 20 bytes per address
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        
        // Create bind group layout
        let bind_group_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("EVM Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        
        let pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("EVM Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        
        let pipeline = self.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("EVM Vanity Search Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("evm_vanity_search"),
            compilation_options: Default::default(),
            cache: None,
        });
        
        let start = Instant::now();
        let max_time = Duration::from_secs(self.config.max_time_secs);
        let mut total_keys = 0u64;
        let mut iteration = 0u32;
        
        loop {
            if stop_flag.load(Ordering::Relaxed) {
                break;
            }
            
            if self.config.max_time_secs > 0 && start.elapsed() > max_time {
                break;
            }
            
            // Create params for this iteration
            let params = SearchParams {
                pattern_len: pattern_len as u32,
                iteration,
                keys_per_thread: keys_per_thread as u32,
                _padding: 0,
            };
            
            let params_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Params Buffer"),
                contents: bytemuck::bytes_of(&params),
                usage: BufferUsages::UNIFORM,
            });
            
            let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("EVM Bind Group"),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: seeds_buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: pattern_buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 2, resource: params_buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 3, resource: results_buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 4, resource: found_keys_buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 5, resource: found_addrs_buffer.as_entire_binding() },
                ],
            });
            
            let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("EVM Compute Encoder"),
            });
            
            {
                let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("EVM Vanity Search Pass"),
                    timestamp_writes: None,
                });
                cpass.set_pipeline(&pipeline);
                cpass.set_bind_group(0, &bind_group, &[]);
                cpass.dispatch_workgroups(num_workgroups, 1, 1);
            }
            
            self.queue.submit(Some(encoder.finish()));
            self.device.poll(wgpu::Maintain::Wait);
            
            // TODO: Read back results and check for matches
            // For now, just count iterations
            
            total_keys += (total_threads * keys_per_thread) as u64;
            iteration += 1;
            
            if iteration % 10 == 0 {
                let elapsed = start.elapsed().as_secs_f64();
                let rate = total_keys as f64 / elapsed / 1_000_000.0;
                debug!(
                    "wgpu: {} keys tested ({:.2} Mkey/s)",
                    total_keys,
                    rate
                );
            }
        }
        
        None
    }
    
    /// Batch pattern matching on GPU (hybrid mode)
    /// 
    /// Takes pre-computed addresses from CPU and finds matches in parallel on GPU.
    /// This is the Phase 1 hybrid approach that works with ALL chains.
    pub fn pattern_match_batch(
        &self,
        addresses: &[String],
        pattern: &str,
        match_type: MatchType,
        case_insensitive: bool,
    ) -> Vec<usize> {
        if addresses.is_empty() || pattern.is_empty() {
            return vec![];
        }
        
        let num_addresses = addresses.len();
        let workgroup_size = 256u32;
        let num_workgroups = ((num_addresses + 255) / 256) as u32;
        
        // Compile pattern match shader
        let shader = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Pattern Match Shader"),
            source: wgpu::ShaderSource::Wgsl(PATTERN_MATCH_SHADER.into()),
        });
        
        // Pack addresses into buffer (64 bytes per address, padded)
        let mut address_data: Vec<u8> = Vec::with_capacity(num_addresses * 64);
        for addr in addresses {
            let bytes = addr.as_bytes();
            let mut padded = [0u8; 64];
            let copy_len = bytes.len().min(64);
            padded[..copy_len].copy_from_slice(&bytes[..copy_len]);
            address_data.extend_from_slice(&padded);
        }
        
        let addresses_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Addresses Buffer"),
            contents: &address_data,
            usage: BufferUsages::STORAGE,
        });
        
        // Pack pattern into buffer
        let pattern_bytes = pattern.as_bytes();
        let mut pattern_data = [0u8; 32];
        let pattern_len = pattern_bytes.len().min(32);
        pattern_data[..pattern_len].copy_from_slice(&pattern_bytes[..pattern_len]);
        
        let pattern_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Pattern Buffer"),
            contents: &pattern_data,
            usage: BufferUsages::STORAGE,
        });
        
        // Params uniform
        #[repr(C)]
        #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
        struct MatchParams {
            pattern_len: u32,
            match_type: u32,
            case_insensitive: u32,
            num_addresses: u32,
        }
        
        let params = MatchParams {
            pattern_len: pattern_len as u32,
            match_type: match_type as u32,
            case_insensitive: if case_insensitive { 1 } else { 0 },
            num_addresses: num_addresses as u32,
        };
        
        let params_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Match Params Buffer"),
            contents: bytemuck::bytes_of(&params),
            usage: BufferUsages::UNIFORM,
        });
        
        // Result buffer
        #[repr(C)]
        #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
        struct MatchResultGpu {
            found: u32,
            first_match_idx: u32,
        }
        
        let result_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Match Result Buffer"),
            size: 8,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        
        // Match flags buffer (one u32 per address)
        let match_flags_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Match Flags Buffer"),
            size: (num_addresses * 4) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        
        // Staging buffer for readback
        let staging_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging Buffer"),
            size: (num_addresses * 4) as u64,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        // Create bind group layout
        let bind_group_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Pattern Match Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 2, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 3, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 4, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None }, count: None },
            ],
        });
        
        let pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pattern Match Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        
        let pipeline = self.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Pattern Match Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("pattern_match"),
            compilation_options: Default::default(),
            cache: None,
        });
        
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Pattern Match Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: pattern_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: addresses_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: params_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: result_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 4, resource: match_flags_buffer.as_entire_binding() },
            ],
        });
        
        // Dispatch compute shader
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Pattern Match Encoder"),
        });
        
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Pattern Match Pass"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&pipeline);
            cpass.set_bind_group(0, &bind_group, &[]);
            cpass.dispatch_workgroups(num_workgroups, 1, 1);
        }
        
        // Copy results to staging buffer
        encoder.copy_buffer_to_buffer(&match_flags_buffer, 0, &staging_buffer, 0, (num_addresses * 4) as u64);
        
        self.queue.submit(Some(encoder.finish()));
        
        // Read back results
        let buffer_slice = staging_buffer.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        self.device.poll(wgpu::Maintain::Wait);
        
        let mut matches = vec![];
        if receiver.recv().unwrap().is_ok() {
            let data = buffer_slice.get_mapped_range();
            let flags: &[u32] = bytemuck::cast_slice(&data);
            for (i, &flag) in flags.iter().enumerate() {
                if flag != 0 {
                    matches.push(i);
                }
            }
        }
        
        matches
    }
    
    /// Benchmark GPU keccak throughput
    pub fn benchmark(&self, duration_secs: u64) -> Result<f64, WgpuError> {
        let workgroup_size = 256u32;
        let num_workgroups = 256u32;
        let total_threads = (workgroup_size * num_workgroups) as usize;
        let keys_per_thread = self.config.keys_per_thread;
        
        // Compile shader
        let shader = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("EVM Benchmark Shader"),
            source: wgpu::ShaderSource::Wgsl(EVM_SHADER.into()),
        });
        
        // Create minimal buffers for benchmark
        let seeds: Vec<[u32; 4]> = (0..total_threads)
            .map(|i| [i as u32, rand::random(), rand::random(), rand::random()])
            .collect();
        
        let seeds_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Seeds Buffer"),
            contents: bytemuck::cast_slice(&seeds),
            usage: BufferUsages::STORAGE,
        });
        
        let pattern_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Pattern Buffer"),
            contents: bytemuck::cast_slice(&[0u32; 4]),
            usage: BufferUsages::STORAGE,
        });
        
        #[repr(C)]
        #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
        struct SearchParams {
            pattern_len: u32,
            iteration: u32,
            keys_per_thread: u32,
            _padding: u32,
        }
        
        let params = SearchParams {
            pattern_len: 0,
            iteration: 0,
            keys_per_thread: keys_per_thread as u32,
            _padding: 0,
        };
        
        let params_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Params Buffer"),
            contents: bytemuck::bytes_of(&params),
            usage: BufferUsages::UNIFORM,
        });
        
        let results_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Results Buffer"),
            size: (total_threads * 16) as u64,
            usage: BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        
        let found_keys_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Found Keys Buffer"),
            size: (total_threads * 32) as u64,
            usage: BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        
        let found_addrs_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Found Addresses Buffer"),
            size: (total_threads * 20) as u64,
            usage: BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        
        // Create bind group layout (same as search)
        let bind_group_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Benchmark Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 2, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 3, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 4, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 5, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None }, count: None },
            ],
        });
        
        let pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Benchmark Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        
        let pipeline = self.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Benchmark Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("evm_benchmark"),
            compilation_options: Default::default(),
            cache: None,
        });
        
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Benchmark Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: seeds_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: pattern_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: params_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: results_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 4, resource: found_keys_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 5, resource: found_addrs_buffer.as_entire_binding() },
            ],
        });
        
        // Warmup
        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let mut cpass = encoder.begin_compute_pass(&Default::default());
            cpass.set_pipeline(&pipeline);
            cpass.set_bind_group(0, &bind_group, &[]);
            cpass.dispatch_workgroups(num_workgroups, 1, 1);
        }
        self.queue.submit(Some(encoder.finish()));
        self.device.poll(wgpu::Maintain::Wait);
        
        // Timed benchmark
        let start = Instant::now();
        let max_time = Duration::from_secs(duration_secs);
        let mut total_keys = 0u64;
        
        while start.elapsed() < max_time {
            let mut encoder = self.device.create_command_encoder(&Default::default());
            {
                let mut cpass = encoder.begin_compute_pass(&Default::default());
                cpass.set_pipeline(&pipeline);
                cpass.set_bind_group(0, &bind_group, &[]);
                cpass.dispatch_workgroups(num_workgroups, 1, 1);
            }
            self.queue.submit(Some(encoder.finish()));
            self.device.poll(wgpu::Maintain::Wait);
            total_keys += (total_threads * keys_per_thread) as u64;
        }
        
        let elapsed = start.elapsed().as_secs_f64();
        let keys_per_second = total_keys as f64 / elapsed;
        
        Ok(keys_per_second)
    }
}

/// List all available wgpu devices
#[cfg(feature = "wgpu-backend")]
pub fn list_wgpu_devices() -> Vec<GpuDevice> {
    pollster::block_on(async {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        
        let adapters: Vec<wgpu::Adapter> = instance.enumerate_adapters(wgpu::Backends::all());
        
        adapters
            .into_iter()
            .enumerate()
            .map(|(i, adapter): (usize, wgpu::Adapter)| {
                let info = adapter.get_info();
                GpuDevice {
                    index: i,
                    name: info.name,
                    compute_capability: format!("{:?}", info.backend),
                    total_memory: 0,
                    multiprocessors: 0,
                    backend: match info.backend {
                        wgpu::Backend::Vulkan => GpuBackend::Vulkan,
                        wgpu::Backend::Metal => GpuBackend::Metal,
                        wgpu::Backend::Dx12 => GpuBackend::Dx12,
                        _ => GpuBackend::Wgpu,
                    },
                }
            })
            .collect()
    })
}

/// Check if wgpu is available
#[cfg(feature = "wgpu-backend")]
pub fn is_wgpu_available() -> bool {
    !list_wgpu_devices().is_empty()
}

#[cfg(not(feature = "wgpu-backend"))]
pub fn list_wgpu_devices() -> Vec<GpuDevice> {
    vec![]
}

#[cfg(not(feature = "wgpu-backend"))]
pub fn is_wgpu_available() -> bool {
    false
}

/// wgpu error type
#[derive(Debug, thiserror::Error)]
pub enum WgpuError {
    #[error("No GPU devices found")]
    NoDevices,
    #[error("Device {0} not found")]
    DeviceNotFound(usize),
    #[error("Device request failed: {0}")]
    DeviceRequest(String),
    #[error("Shader compilation failed")]
    ShaderCompilation,
}
