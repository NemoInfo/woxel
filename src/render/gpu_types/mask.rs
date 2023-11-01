use crate::{
    render::gpu_types::GpuUniform,
    vdb::{VdbValueType, VDB345},
};
use wgpu::{util::DeviceExt, BindGroup, BindGroupLayout, Buffer, Device};

#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Clone)]
pub struct MaskUniform {
    buff: Vec<u8>,
}

impl GpuUniform for MaskUniform {
    fn bind(&self, device: &Device) -> (Buffer, Vec<u8>, BindGroup, BindGroupLayout) {
        let buffer_contents = self.get_buffer_contents();
        let buffer = self.create_buffer(device, &buffer_contents);
        let layout = self.create_bind_group_layout(device);
        let bind_group = self.create_bind_group(&buffer, &layout, device);

        (buffer, buffer_contents, bind_group, layout)
    }
}

impl MaskUniform {
    fn get_buffer_contents(&self) -> Vec<u8> {
        self.buff.clone()
    }

    fn create_buffer(&self, device: &Device, buffer_contents: &[u8]) -> Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mask Buffer"),
            contents: buffer_contents,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    }

    fn create_bind_group_layout(&self, device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("Mask Bind Group Layout"),
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
            label: Some("Mask Bind Group"),
        })
    }
}

impl<'a, ValueType: VdbValueType> From<&'a VDB345<ValueType>> for MaskUniform {
    fn from(vdb: &'a VDB345<ValueType>) -> Self {
        // HACK: This is a weird way to do this maybe tghis should be done in  the bind method
        let masks = vdb.masks();
        let mut buff = Vec::new();
        for vec in &masks {
            for &num in vec {
                buff.push((num >> 24) as u8); // Extract the first byte
                buff.push((num >> 16) as u8); // Extract the second byte
                buff.push((num >> 8) as u8); // Extract the third byte
                buff.push(num as u8); // Extract the fourth byte
            }
        }
        Self { buff }
    }
}
