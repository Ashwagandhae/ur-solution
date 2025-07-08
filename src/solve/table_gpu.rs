use itertools::Itertools;
use rayon::prelude::*;
use rustc_hash::FxHashMap as HashMap;
/// To serve as an introduction to the wgpu api, we will implement a simple
/// compute shader which takes a list of numbers on the CPU and doubles them on the GPU.
///
/// While this isn't a very practical example, you will see all the major components
/// of using wgpu headlessly, including getting a device, running a shader, and transferring
/// data between the CPU and GPU.
///
/// If you time the recording and execution of this example you will certainly see that
/// running on the gpu is slower than doing the same calculation on the cpu. This is because
/// floating point multiplication is a very simple operation so the transfer/submission overhead
/// is quite a lot higher than the actual computation. This is normal and shows that the GPU
/// needs a lot higher work/transfer ratio to come out ahead.
use std::num::NonZeroU64;
use wgpu::util::DeviceExt;

use crate::{game::GameState, solve::expr::get_exprs};

const VALS_DIVIDER: usize = 4;
pub struct TableGpu {
    device: wgpu::Device,
    queue: wgpu::Queue,

    pub vals: Vec<f32>,
    pub old_vals: Vec<f32>,

    old_val_buffer: wgpu::Buffer,
    val_buffer: wgpu::Buffer,
    val_buffer_size: u64,
    param_buffer: wgpu::Buffer,
    download_val_buffer: wgpu::Buffer,
    download_old_val_buffer: wgpu::Buffer,
    expr_parts_1_buffer: wgpu::Buffer,
    expr_parts_2_buffer: wgpu::Buffer,
    expr_starts_buffer: wgpu::Buffer,

    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,

    workgroups_x: u32,
    workgroups_y: u32,
}

impl TableGpu {
    pub fn new(state_indices: &HashMap<GameState, u32>, states: &[GameState]) -> Self {
        let (expr_parts, expr_starts) = get_exprs(state_indices, states);
        println!("number of expr parts: {}", expr_parts.len());

        // We first initialize an wgpu `Instance`, which contains any "global" state wgpu needs.
        //
        // This is what loads the vulkan/dx12/metal/opengl libraries.
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());

        // We then create an `Adapter` which represents a physical gpu in the system. It allows
        // us to query information about it and create a `Device` from it.
        //
        // This function is asynchronous in WebGPU, so request_adapter returns a future. On native/webgl
        // the future resolves immediately, so we can block on it without harm.
        let adapter =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
                .expect("Failed to create adapter");

        // Print out some basic information about the adapter.

        // Check to see if the adapter supports compute shaders. While WebGPU guarantees support for
        // compute shaders, wgpu supports a wider range of devices through the use of "downlevel" devices.
        let downlevel_capabilities = adapter.get_downlevel_capabilities();
        if !downlevel_capabilities
            .flags
            .contains(wgpu::DownlevelFlags::COMPUTE_SHADERS)
        {
            panic!("Adapter does not support compute shaders");
        }

        // We then create a `Device` and a `Queue` from the `Adapter`.
        //
        // The `Device` is used to create and manage GPU resources.
        // The `Queue` is a queue used to submit work for the GPU to process.

        let mut required_limits = wgpu::Limits::downlevel_defaults();
        required_limits.max_buffer_size = 8589934592;
        required_limits.max_storage_buffers_per_shader_stage = 5;
        let required_features = wgpu::Features::empty();
        required_limits.max_storage_buffer_binding_size = u32::MAX;
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: None,
            required_features,
            required_limits,
            memory_hints: wgpu::MemoryHints::MemoryUsage,
            trace: wgpu::Trace::Off,
        }))
        .expect("Failed to create device");

        // Create a shader module from our shader code. This will parse and validate the shader.
        //
        // `include_wgsl` is a macro provided by wgpu like `include_str` which constructs a ShaderModuleDescriptor.
        // If you want to load shaders differently, you can construct the ShaderModuleDescriptor manually.
        let module = device.create_shader_module(wgpu::include_wgsl!("expr.wgsl"));

        let states_section_len = states.len() / VALS_DIVIDER;
        let val_buffer_size = (states_section_len * std::mem::size_of::<f32>()) as u64;

        let total_invocations = states_section_len;
        let workgroup_size = 256;
        let total_workgroups = (total_invocations + workgroup_size - 1) / workgroup_size;

        let max_x = 65535;
        let workgroups_x = max_x.min(total_workgroups as u32);
        let workgroups_y = ((total_workgroups as f32) / workgroups_x as f32).ceil() as u32;
        let width = workgroups_x as usize * workgroup_size;

        let param_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Params Buffer"),
            contents: bytemuck::bytes_of(&Params {
                width: width as u32,
            }),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let val_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Val Buffer"),
            size: val_buffer_size,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let old_val_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Old Val Buffer"),
            size: val_buffer_size,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let expr_parts_num = expr_parts.iter().cloned().map(u32::from).collect_vec();
        let expr_parts_cutoff = expr_parts_num.len() / 2;
        let expr_parts_1_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Expr Parts Buffer 1"),
            contents: bytemuck::cast_slice(&expr_parts_num[..expr_parts_cutoff]),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let expr_parts_2_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Expr Parts Buffer 2"),
            contents: bytemuck::cast_slice(&expr_parts_num[expr_parts_cutoff..]),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let expr_starts_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Expr Starts Buffer"),
            contents: bytemuck::cast_slice(&expr_starts),
            usage: wgpu::BufferUsages::STORAGE,
        });

        // Copy the final result to a readable buffer
        let download_val_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Download Val Buffer"),
            size: val_buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let download_old_val_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Download Old Val Buffer"),
            size: val_buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(NonZeroU64::new(4).unwrap()),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: Some(NonZeroU64::new(4).unwrap()),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: Some(NonZeroU64::new(4).unwrap()),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: Some(NonZeroU64::new(4).unwrap()),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: Some(NonZeroU64::new(4).unwrap()),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: Some(NonZeroU64::new(4).unwrap()),
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Eval Expr Pipeline"),
            layout: Some(&pipeline_layout),
            module: &module,
            entry_point: Some("evalExpr"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        let vals = vec![0.0; states.len()];
        let old_vals = vec![0.0; states.len()];

        Self {
            device,
            queue,
            bind_group_layout,
            val_buffer,
            old_val_buffer,
            param_buffer,
            download_val_buffer,
            download_old_val_buffer,
            pipeline,
            workgroups_x,
            workgroups_y,
            val_buffer_size,
            expr_parts_1_buffer,
            expr_parts_2_buffer,
            expr_starts_buffer,
            vals,
            old_vals,
        }
    }

    pub fn converge(&mut self, n: usize) {
        std::mem::swap(&mut self.vals, &mut self.old_vals);

        let Self {
            device,
            queue,

            val_buffer,
            old_val_buffer,
            param_buffer,

            pipeline,
            bind_group_layout,
            workgroups_x,
            workgroups_y,

            expr_parts_1_buffer,
            expr_parts_2_buffer,
            expr_starts_buffer,
            ..
        } = self;
        // Ping-pong buffers

        for _ in 0..n {
            std::mem::swap(old_val_buffer, val_buffer);

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: param_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: old_val_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: val_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: expr_starts_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: expr_parts_1_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: expr_parts_2_buffer.as_entire_binding(),
                    },
                ],
            });

            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Compute Encoder"),
            });

            {
                let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Compute Pass"),
                    timestamp_writes: None,
                });

                compute_pass.set_pipeline(&pipeline);
                compute_pass.set_bind_group(0, &bind_group, &[]);
                compute_pass.dispatch_workgroups(*workgroups_x, *workgroups_y, 1);
            }

            queue.submit(Some(encoder.finish()));
            device.poll(wgpu::PollType::Wait).unwrap();
        }
    }

    pub fn stats(&mut self) {
        // Record both copies
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Final Copy Encoder"),
            });
        encoder.copy_buffer_to_buffer(
            &self.old_val_buffer,
            0,
            &self.download_old_val_buffer,
            0,
            self.val_buffer_size,
        );
        encoder.copy_buffer_to_buffer(
            &self.val_buffer,
            0,
            &self.download_val_buffer,
            0,
            self.val_buffer_size,
        );
        self.queue.submit(Some(encoder.finish()));
        self.device.poll(wgpu::PollType::Wait).unwrap();

        // Map both buffers
        let slice_input = self.download_old_val_buffer.slice(..);
        let slice_output = self.download_val_buffer.slice(..);
        slice_input.map_async(wgpu::MapMode::Read, |_| {});
        slice_output.map_async(wgpu::MapMode::Read, |_| {});
        self.device.poll(wgpu::PollType::Wait).unwrap();

        // Read both into vectors
        let data_input = slice_input.get_mapped_range();
        let data_output = slice_output.get_mapped_range();

        self.old_vals = bytemuck::cast_slice(&data_input).to_vec();
        self.vals = bytemuck::cast_slice(&data_output).to_vec();

        drop(data_input);
        drop(data_output);
        self.download_old_val_buffer.unmap();
        self.download_val_buffer.unmap();

        let total_delta: f32 = self
            .vals
            .par_iter()
            .zip(self.old_vals.par_iter())
            .map(|(x, y)| (x - y).abs())
            .sum();

        let max_delta: f32 = self
            .vals
            .par_iter()
            .zip(self.old_vals.par_iter())
            .map(|(x, y)| (x - y).abs())
            .reduce(|| f32::NEG_INFINITY, f32::max);

        println!(
            "total delta: {}, average delta: {}, max delta: {}",
            total_delta,
            total_delta / self.vals.len() as f32,
            max_delta
        );
    }

    pub fn vals(&self) -> &[f32] {
        &self.vals
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
struct Params {
    width: u32,
}

unsafe impl bytemuck::Pod for Params {}
unsafe impl bytemuck::Zeroable for Params {}
