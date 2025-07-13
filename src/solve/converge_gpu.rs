use itertools::Itertools;
use rayon::prelude::*;
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

use crate::solve::expr::ExprPart;

pub struct DeviceHolder {
    device: wgpu::Device,
    queue: wgpu::Queue,

    bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::ComputePipeline,
}

impl DeviceHolder {
    pub fn new() -> Self {
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
        let module = device.create_shader_module(wgpu::include_wgsl!("converge.wgsl"));

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    // params
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(NonZeroU64::new(8).unwrap()),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    // dep_vals
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
                    // in_vals
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: Some(NonZeroU64::new(4).unwrap()),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    // out_vals
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: Some(NonZeroU64::new(4).unwrap()),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    // expr_starts
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
                    // expr_starts
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

        Self {
            device,
            queue,
            bind_group_layout,
            pipeline,
        }
    }
}

pub struct Converger {
    param_buffer: wgpu::Buffer,
    dep_vals_buffer: wgpu::Buffer,
    old_vals_buffer: wgpu::Buffer,
    vals_buffer: wgpu::Buffer,
    expr_starts_buffer: wgpu::Buffer,
    expr_parts_buffer: wgpu::Buffer,

    download_vals_buffer: wgpu::Buffer,
    download_old_vals_buffer: wgpu::Buffer,

    workgroups_x: u32,
    workgroups_y: u32,
}

impl Converger {
    pub fn new(
        device_holder: &mut DeviceHolder,
        dep_start: usize,

        dep_vals: &[f32],
        in_vals_len: usize,

        expr_parts: &[ExprPart],
        expr_starts: &[u32],
    ) -> Self {
        let DeviceHolder { device, .. } = device_holder;

        let val_buffer_size = (in_vals_len * std::mem::size_of::<f32>()) as u64;

        let total_invocations = in_vals_len;
        let workgroup_size = 256;
        let total_workgroups = total_invocations.div_ceil(workgroup_size);

        let max_x = 65535;
        let workgroups_x = max_x.min(total_workgroups as u32);
        let workgroups_y = ((total_workgroups as f32) / workgroups_x as f32).ceil() as u32;
        let width = workgroups_x as usize * workgroup_size;

        let param_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Params Buffer"),
            contents: bytemuck::bytes_of(&Params {
                width: width as u32,
                dep_start: dep_start as u32,
            }),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let dep_vals = if dep_vals.is_empty() {
            &vec![0.0]
        } else {
            dep_vals
        };

        let dep_vals_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Dep Vals Buffer"),
            contents: bytemuck::cast_slice(dep_vals),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let init_vals: Vec<f32> = vec![0.0; in_vals_len];
        let vals_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vals Buffer"),
            contents: bytemuck::cast_slice(&init_vals),
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
        });
        let old_vals_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vals Buffer"),
            contents: bytemuck::cast_slice(&init_vals),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        });

        let expr_parts = expr_parts.iter().copied().map(u32::from).collect_vec();
        let expr_parts_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Expr Parts Buffer"),
            contents: bytemuck::cast_slice(&expr_parts),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let expr_starts_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Expr Starts Buffer"),
            contents: bytemuck::cast_slice(expr_starts),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let download_vals_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Download Val Buffer"),
            size: val_buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let download_old_vals_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Download Old Val Buffer"),
            size: val_buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        Self {
            param_buffer,
            dep_vals_buffer,
            old_vals_buffer,
            vals_buffer,
            expr_starts_buffer,
            expr_parts_buffer,

            download_vals_buffer,
            download_old_vals_buffer,

            workgroups_x,
            workgroups_y,
        }
    }
    pub fn converge(
        &mut self,
        device_holder: &mut DeviceHolder,
        n: usize,
        in_vals: &mut [f32],
        out_vals: &mut [f32],
    ) {
        let Self {
            param_buffer,
            dep_vals_buffer,
            vals_buffer,
            old_vals_buffer,
            expr_starts_buffer,
            expr_parts_buffer,
            workgroups_x,
            workgroups_y,
            download_old_vals_buffer,
            download_vals_buffer,
            ..
        } = self;

        let DeviceHolder {
            device,
            bind_group_layout,
            pipeline,
            queue,
            ..
        } = device_holder;
        for _ in 0..n {
            std::mem::swap(old_vals_buffer, vals_buffer);

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: param_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: dep_vals_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: vals_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: old_vals_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: expr_starts_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: expr_parts_buffer.as_entire_binding(),
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

                compute_pass.set_pipeline(pipeline);
                compute_pass.set_bind_group(0, &bind_group, &[]);
                compute_pass.dispatch_workgroups(*workgroups_x, *workgroups_y, 1);
            }

            queue.submit(Some(encoder.finish()));
            device.poll(wgpu::PollType::Wait).unwrap();
        }
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Final Copy Encoder"),
        });
        encoder.copy_buffer_to_buffer(
            old_vals_buffer,
            0,
            download_old_vals_buffer,
            0,
            old_vals_buffer.size(),
        );
        encoder.copy_buffer_to_buffer(vals_buffer, 0, download_vals_buffer, 0, vals_buffer.size());
        queue.submit(Some(encoder.finish()));
        device.poll(wgpu::PollType::Wait).unwrap();

        // Map both buffers
        let slice_input = download_old_vals_buffer.slice(..);
        let slice_output = download_vals_buffer.slice(..);
        slice_input.map_async(wgpu::MapMode::Read, |_| {});
        slice_output.map_async(wgpu::MapMode::Read, |_| {});
        device.poll(wgpu::PollType::Wait).unwrap();

        // Read both into vectors
        let data_input = slice_input.get_mapped_range();
        let data_output = slice_output.get_mapped_range();

        in_vals.copy_from_slice(bytemuck::cast_slice(&data_input));
        out_vals.copy_from_slice(bytemuck::cast_slice(&data_output));

        drop(data_input);
        drop(data_output);
        download_old_vals_buffer.unmap();
        download_vals_buffer.unmap();
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
struct Params {
    width: u32,
    dep_start: u32,
}

unsafe impl bytemuck::Pod for Params {}
unsafe impl bytemuck::Zeroable for Params {}
