use std::collections::HashMap;

use log::warn;
use wgpu::{BindGroup, BindGroupLayout, ComputePipeline, ShaderModule, Texture};
use winit::window::Window;

use crate::scene::Scene;

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

        Self {
            surface,
            device,
            queue,
            config,
            size,
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

        let (camera_buffer, camera_buffer_contents, camera_bind_group, camera_bind_group_layout) =
            FrameDescriptor::create_camera_binding(&scene.camera, &self.device);

        let (ray_buffer, ray_buffer_contents, ray_bind_group, ray_bind_group_layout) =
            FrameDescriptor::create_ray_binding(&scene.camera, self.size, &self.device);

        let (texture, texture_view, texture_bind_group, texture_bind_group_layout) =
            FrameDescriptor::create_output_texture_binding(&self.device);

        let compute_pipline_layout =
            self.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Compute Pipline Layout"),
                    bind_group_layouts: &[
                        &camera_bind_group_layout,
                        &ray_bind_group_layout,
                        &texture_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });

        let fragment_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: 1600,
                height: 900,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("Fragment Texture"),
            view_formats: &[],
        });

        let fragment_texture_view = fragment_texture.create_view(&wgpu::TextureViewDescriptor {
            ..Default::default()
        });

        let fragment_texture_sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let fragment_texture_bind_group_layout =
            self.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Fragment Texture Bind Group Layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });

        let fragment_texture_bind_group =
            self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Fragment Texture Bind Group"),
                layout: &fragment_texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&fragment_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&fragment_texture_sampler),
                    },
                ],
            });

        let render_pipeline_layout =
            self.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[&fragment_texture_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let compute_pipeline = ComputePipeline.get(self, compute_pipline_layout);
        let render_pipeline = VoxelPipeline.get(self, render_pipeline_layout);

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute Pass"),
            });

            compute_pass.set_pipeline(&compute_pipeline);
            compute_pass.set_bind_group(0, &camera_bind_group, &[]);
            compute_pass.set_bind_group(1, &ray_bind_group, &[]);
            compute_pass.set_bind_group(2, &texture_bind_group, &[]);
            compute_pass.dispatch_workgroups(1600, 900, 1);
        }

        encoder.copy_texture_to_texture(
            texture.as_image_copy(),
            fragment_texture.as_image_copy(),
            wgpu::Extent3d {
                width: 1600,
                height: 900,
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
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..num_indices, 0, 0..1);
        }

        self.queue
            .write_buffer(&camera_buffer, 0, &camera_buffer_contents);

        self.queue
            .write_buffer(&ray_buffer, 0, &ray_buffer_contents);

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
            .unwrap_or_else(|| panic!("No shader with name '{name}"))
    }
}
