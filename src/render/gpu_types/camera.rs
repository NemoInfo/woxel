use crate::render::{gpu_types::GpuUniform, Camera};
use bytemuck_derive::{Pod, Zeroable};
use cgmath::SquareMatrix;
use wgpu::{util::DeviceExt, BindGroup, BindGroupLayout, Buffer, Device};

#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct CameraUniform {
    // We can't use cgmath with bytemuck directly so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    view_projection: [[f32; 4]; 4],
    camera_to_world: [[f32; 4]; 4],
}

impl GpuUniform for CameraUniform {
    fn bind(&self, device: &Device) -> (Buffer, Vec<u8>, wgpu::BindGroup, BindGroupLayout) {
        let buffer_contents = self.get_buffer_contents();
        let buffer = self.create_buffer(device, &buffer_contents);
        let layout = self.create_bind_group_layout(device);
        let bind_group = self.create_bind_group(&buffer, &layout, device);

        (buffer, buffer_contents, bind_group, layout)
    }
}

impl CameraUniform {
    fn get_buffer_contents(&self) -> Vec<u8> {
        bytemuck::cast_slice(&[*self]).to_vec()
    }

    fn create_buffer(&self, device: &Device, buffer_contents: &[u8]) -> Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
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
            label: Some("Camera Bind Group Layout"),
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
            label: Some("Camera Bind Group"),
        })
    }
}

impl<'a> From<&'a Camera> for CameraUniform {
    fn from(c: &'a Camera) -> Self {
        let view_projection = c.build_view_projection_matrix();
        let camera_to_world = (match view_projection.invert() {
            Some(c) => c,
            None => panic!("Could not invert camera matrix"),
        })
        .into();
        let view_projection = view_projection.into();

        Self {
            view_projection,
            camera_to_world,
        }
    }
}
