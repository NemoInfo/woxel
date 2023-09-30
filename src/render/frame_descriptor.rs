use std::num::NonZeroU32;

use wgpu::{
    util::DeviceExt, BindGroup, BindGroupLayout, Buffer, Color, Device, Sampler, Texture,
    TextureView,
};
use winit::dpi::PhysicalSize;

use crate::render::{
    gpu_types::{CameraUniform, GpuUniform},
    Camera,
};

use super::gpu_types::{GpuPrimitive, GpuQuad, RayUniform, GPU_QUAD};

pub struct FrameDescriptor {
    pub clear_color: Color,
}

impl FrameDescriptor {
    pub fn build() -> Self {
        let clear_color = wgpu::Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };

        FrameDescriptor { clear_color }
    }

    pub fn create_camera_binding(
        camera: &Camera,
        device: &Device,
    ) -> (Buffer, Vec<u8>, BindGroup, BindGroupLayout) {
        CameraUniform::from(camera).bind(device)
    }

    pub fn create_vertex_buffer(&self, device: &Device) -> Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: &GPU_QUAD.data(),
            usage: wgpu::BufferUsages::VERTEX,
        })
    }

    pub fn indicies(&self) -> Vec<u16> {
        GpuQuad::INDEXES.to_vec()
    }

    pub fn create_index_buffer(&self, device: &Device) -> Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&self.indicies()),
            usage: wgpu::BufferUsages::INDEX,
        })
    }

    pub fn create_ray_binding(
        camera: &Camera,
        size: PhysicalSize<u32>,
        device: &Device,
    ) -> (Buffer, Vec<u8>, BindGroup, BindGroupLayout) {
        RayUniform::build(&camera, size.width as f32).bind(device)
    }

    pub fn create_output_texture_binding(
        device: &Device,
    ) -> (Texture, TextureView, BindGroup, BindGroupLayout) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: 1600,
                height: 900,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
            label: Some("Compute Texture"),
            view_formats: &[],
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Compute Texture Output"),
            format: Some(wgpu::TextureFormat::Rgba8Unorm),
            base_mip_level: 0,
            mip_level_count: Some(1),
            ..Default::default()
        });

        let bind_group_layput = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Compute Texture Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute Texture Bind Group"),
            layout: &bind_group_layput,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            }],
        });

        (texture, texture_view, bind_group, bind_group_layput)
    }
}
