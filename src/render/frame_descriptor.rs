use wgpu::{util::DeviceExt, BindGroup, BindGroupLayout, Buffer, Color, Device};
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
}
