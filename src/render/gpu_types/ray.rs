use bytemuck_derive::{Pod, Zeroable};
use cgmath::SquareMatrix;
use wgpu::{util::DeviceExt, BindGroup, BindGroupLayout, Buffer, Device};

use crate::render::Camera;

use super::GpuUniform;

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct RayUniform {
    // First row of camera to world matrix (u)
    u: [f32; 4],
    // Negative second row of camera to world matrix (-v)
    mv: [f32; 4],
    // w' = (-width / 2) u + (height / 2) v - ((height / 2) / tan(fov * 0.5)) w
    wp: [f32; 4],
}

impl GpuUniform for RayUniform {
    fn bind(&self, device: &Device) -> (Buffer, Vec<u8>, BindGroup, BindGroupLayout) {
        let buffer_contents = self.get_buffer_contents();
        let buffer = self.create_buffer(device, &buffer_contents);
        let layout = self.create_bind_group_layout(device);
        let bind_group = self.create_bind_group(&buffer, &layout, device);

        (buffer, buffer_contents, bind_group, layout)
    }
}

impl<'a> RayUniform {
    fn get_buffer_contents(&self) -> Vec<u8> {
        bytemuck::cast_slice(&[*self]).to_vec()
    }

    fn create_buffer(&self, device: &Device, buffer_contents: &[u8]) -> Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Ray Buffer"),
            contents: buffer_contents,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    }

    fn create_bind_group_layout(&self, device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("Ray Bind Group Layout"),
        })
    }

    fn create_bind_group(
        &self,
        buffer: &Buffer,
        layout: &BindGroupLayout,
        device: &Device,
    ) -> BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("Ray Bind Group"),
        })
    }

    pub fn build(c: &'a Camera, resolution_width: f32) -> Self {
        let view_proj = c.build_view_projection_matrix();
        let camera_to_world = match view_proj.invert() {
            Some(c) => c,
            None => panic!("Could not invert camera matrix"),
        };
        // @TODO: Thorough check this !!! No chance I got it right the first time
        let height = resolution_width / c.aspect;
        let u = camera_to_world.x;
        let v = camera_to_world.y;
        let w = camera_to_world.z;
        let wp = (-resolution_width / 2.0) * u + (height / 2.0) * v
            - w * (height / 2.0) / (c.fovy.to_radians() * 0.5).tan();
        let mv = -v;

        Self {
            u: u.into(),
            mv: mv.into(),
            wp: wp.into(),
        }
    }
}
