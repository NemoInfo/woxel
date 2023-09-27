use wgpu::{util::DeviceExt, BindGroup, BindGroupLayout, Buffer, Color, Device};

use crate::render::{
    gpu_types::{CameraUniform, GpuUniform},
    Camera,
};

use super::gpu_types::{GpuPrimitive, GpuQuad, RayUniform, GPU_QUAD};

pub struct FrameDescriptor {
    camera: Camera,
    pub clear_color: Color,
}

impl FrameDescriptor {
    pub fn build() -> Self {
        let camera = Camera::quick_camera(1600.0 / 900.0);

        let clear_color = wgpu::Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };

        FrameDescriptor {
            camera,
            clear_color,
        }
    }

    pub fn create_camera_binding(
        &self,
        device: &Device,
    ) -> (Buffer, Vec<u8>, BindGroup, BindGroupLayout) {
        CameraUniform::from(&self.camera).bind(device)
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
        &self,
        device: &Device,
    ) -> (Buffer, Vec<u8>, BindGroup, BindGroupLayout) {
        RayUniform::build(&self.camera, 1600.0).bind(device)
    }
}
