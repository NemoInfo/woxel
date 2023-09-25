use std::collections::HashMap;

use tracing::info;
use wgpu::{BindGroup, BindGroupLayout, ShaderModule, Texture};
use winit::window::Window;

use super::{
    frame_descriptor::FrameDescriptor,
    pipelines::{Pipeline, VoxelPipeline},
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
        println!("Backend: {}", backend);
        println!("Adaptor: {}", name);

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

        println!("Device: {:?}", &device);

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

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
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
            frame_descriptor.create_camera_binding(&self.device);

        let render_pipeline_layout =
            self.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[&camera_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let render_pipeline = VoxelPipeline.get(self, render_pipeline_layout);

        // let instance_buffer = frame_descriptor.create_instance_buffer(&self.device);

        // let num_instances = frame_descriptor.instances().len() as u32;

        // let particle_pipeline_layout =
        //     self.device
        //         .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        //             label: Some("Solid Pipline Layout"),
        //             bind_group_layouts: &[&camera_bind_group_layout],
        //             push_constant_ranges: &[],
        //         });
        // let particle_pipeline = ParticlePipeline.get(self, particle_pipeline_layout);

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
            render_pass.set_bind_group(0, &camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..num_indices, 0, 0..1);
        }

        self.queue
            .write_buffer(&camera_buffer, 0, &camera_buffer_contents);

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
