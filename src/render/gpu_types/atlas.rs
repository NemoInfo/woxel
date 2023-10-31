use wgpu::{BindGroup, BindGroupLayout, Device, Texture, TextureView};

use super::GpuTexture;

pub struct NodeAtlas {
    size: [u32; 3],
}

impl GpuTexture for NodeAtlas {
    fn bind(&self, device: &Device) -> (Texture, BindGroup, BindGroupLayout) {
        let texture = self.create_texture(device);
        let view = self.create_texture_view(&texture);
        let bind_group_layout = self.create_bind_group_layout(device);
        let bind_group = self.create_bind_group(device, &bind_group_layout, &view);

        (texture, bind_group, bind_group_layout)
    }
}

impl NodeAtlas {
    pub fn new(size: [u32; 3]) -> Self {
        Self { size }
    }

    fn create_texture(&self, device: &Device) -> Texture {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: self.size[0],
                height: self.size[1],
                depth_or_array_layers: self.size[2],
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D3,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
            label: Some("Atlas Texture"),
            view_formats: &[],
        });

        texture
    }

    fn create_texture_view(&self, texture: &Texture) -> TextureView {
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Atlas Texture View"),
            format: Some(wgpu::TextureFormat::Rgba8Unorm),
            base_mip_level: 0,
            mip_level_count: Some(1),
            ..Default::default()
        });

        texture_view
    }

    fn create_bind_group_layout(&self, device: &Device) -> BindGroupLayout {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Atlas Texture Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::ReadOnly,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    view_dimension: wgpu::TextureViewDimension::D3,
                },
                count: None,
            }],
        });

        bind_group_layout
    }

    fn create_bind_group(
        &self,
        device: &Device,
        layout: &BindGroupLayout,
        view: &TextureView,
    ) -> BindGroup {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Atlas Texture Bind Group"),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(view),
            }],
        });

        bind_group
    }
}
