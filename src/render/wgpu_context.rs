use std::{collections::HashMap, io::BufReader};

use log::warn;
use wgpu::{BindGroup, BindGroupLayout, ShaderModule, Texture};
use winit::window::Window;

use crate::{scene::Scene, vdb::VdbReader};

use super::{
    frame_descriptor::FrameDescriptor,
    pipelines::{CPipeline, ComputePipeline, Pipeline, VoxelPipeline},
};

pub struct WgpuContext {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    atlas_group: ([Texture; 3], BindGroup, BindGroupLayout),
    shaders: HashMap<&'static str, ShaderModule>,
    _textures: HashMap<&'static str, (Texture, BindGroup, BindGroupLayout)>,
}

impl WgpuContext {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let backend = format!("{:?}", adapter.get_info().backend);
        let name = format!("{:?}", adapter.get_info().name);
        warn!("Backend: {}", backend);
        warn!("Adaptor: {}", name);

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();

        warn!("Device: {:?}", &device);

        let limits = device.limits();
        println!("Maximum bind groups supported: {}", limits.max_bind_groups);

        let surface_caps = surface.get_capabilities(&adapter);
        // we want to work with srgb format
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .filter(|f| f.is_srgb())
            .next()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let f = std::fs::File::open("assets/utahteapot.vdb").unwrap();
        let mut vdb_reader = VdbReader::new(BufReader::new(f)).unwrap();
        let vdb = vdb_reader.read_vdb345_grid::<f32>("ls_utahteapot").unwrap();
        println!("Loaded vdb");
        let atlas = vdb.atlas();

        let atlas_size @ [size5, size4, size3] =
            atlas.iter().map(|n| [n.len() as u32, n[0].len() as u32, n[0][0].len() as u32]).collect::<Vec<_>>().try_into().unwrap();

        dbg!(atlas_size);

        let ([texture5, texture4, texture3], bind_group, bind_group_layout) =
            FrameDescriptor::create_compute_vdb_atlas_texture_binding(&device, atlas_size);

        let flat_atlas: [Vec<u8>; 3] = atlas.iter().map(|n| {
            n.iter()
                .flat_map(|plane| {
                    plane
                        .iter()
                        .flat_map(|row| row.iter().flat_map(|val| val.to_ne_bytes()))
                })
                .collect()
        }).collect::<Vec<_>>().try_into().unwrap();

        queue.write_texture(
            texture5.as_image_copy(),
            &flat_atlas[0],
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(size5[0] * 4), // 4 bytes per u32
                rows_per_image: Some(size5[1]),
            },
            wgpu::Extent3d {
                width: size5[0],
                height: size5[1],
                depth_or_array_layers: size5[2],
            },
        );

        println!("Wrote texture5");

        queue.write_texture(
            texture4.as_image_copy(),
            &flat_atlas[1],
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(size4[0] * 4), // 4 bytes per u32
                rows_per_image: Some(size4[1]),
            },
            wgpu::Extent3d {
                width: size4[0],
                height: size4[1],
                depth_or_array_layers: size4[2],
            },
        );

        println!("Wrote texture4");

        queue.write_texture(
            texture3.as_image_copy(),
            &flat_atlas[2],
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(size3[0] * 4), // 4 bytes per u32
                rows_per_image: Some(size3[1]),
            },
            wgpu::Extent3d {
                width: size3[0],
                height: size3[1],
                depth_or_array_layers: size3[2],
            },
        );

        println!("Wrote texture3");

        Self {
            surface,
            device,
            queue,
            config,
            size,
            atlas_group: ([texture5, texture4, texture3], bind_group, bind_group_layout),
            shaders: HashMap::new(),
            _textures: HashMap::new(),
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn render(&mut self, scene: &Scene) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let frame_descriptor = FrameDescriptor::build();

        let vertex_buffer = frame_descriptor.create_vertex_buffer(&self.device);

        let index_buffer = frame_descriptor.create_index_buffer(&self.device);

        let num_indices = frame_descriptor.indicies().len() as u32;

        let (compute_state_buffer, compute_state_buffer_contents, compute_state_bind_group, compute_state_bind_group_layout) =
            FrameDescriptor::create_compute_state_binding(&scene.camera, self.size, &self.device);

        let (state_buffer, state_buffer_contents, state_bind_group, state_bind_group_layout) =
            FrameDescriptor::create_screen_state_binding(&self.device, &scene.state);

        let (compute_texture, compute_texture_bind_group, compute_texture_bind_group_layout) =
            FrameDescriptor::create_compute_output_texture_binding(&self.device, self.size.into());

        let (fragment_texture, fragment_texture_bind_group, fragment_texture_bind_group_layout) =
            FrameDescriptor::create_fragment_texture_binding(&self.device, self.size.into());


        let compute_pipline_layout =
            self.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Compute Pipline Layout"),
                    bind_group_layouts: &[
                        &compute_state_bind_group_layout,
                        &compute_texture_bind_group_layout,
                        &self.atlas_group.2,
                    ],
                    push_constant_ranges: &[],
                });

        let render_pipeline_layout =
            self.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[
                        &fragment_texture_bind_group_layout,
                        &state_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });

        let compute_pipeline = ComputePipeline.get(self, compute_pipline_layout);
        let render_pipeline = VoxelPipeline.get(self, render_pipeline_layout);

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute Pass"),
            });

            compute_pass.set_pipeline(&compute_pipeline);
            compute_pass.set_bind_group(0, &compute_state_bind_group, &[]);
            compute_pass.set_bind_group(1, &compute_texture_bind_group, &[]);
            compute_pass.set_bind_group(2, &self.atlas_group.1, &[]);
            // @TODO: CHOOSE WORKGROUPS BASED ON ADAPTOR (32 for NVDIA, 64 for AMD)
            compute_pass.dispatch_workgroups(self.size.width / 8, self.size.height / 4, 1);
        }

        encoder.copy_texture_to_texture(
            compute_texture.as_image_copy(),
            fragment_texture.as_image_copy(),
            wgpu::Extent3d {
                width: self.size.width,
                height: self.size.height,
                depth_or_array_layers: 1,
            },
        );

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(frame_descriptor.clear_color),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(&render_pipeline);
            render_pass.set_bind_group(0, &fragment_texture_bind_group, &[]);
            render_pass.set_bind_group(1, &state_bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..num_indices, 0, 0..1);
        }

        self.queue
            .write_buffer(&state_buffer, 0, &state_buffer_contents);

        self.queue
            .write_buffer(&compute_state_buffer, 0, &compute_state_buffer_contents);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn add_shader(&mut self, name: &'static str, source: &'static str) {
        if self.shaders.contains_key(name) {
            panic!("Shader with name '{name}' already exists");
        }
        let shader = crate::render::Shader::new(name, source);
        self.shaders.insert(name, shader.bind(&self.device));
    }

    pub fn get_shader(&self, name: &'static str) -> &ShaderModule {
        self.shaders
            .get(name)
            .unwrap_or_else(|| panic!("No shader with name '{name}'"))
    }
}
