use wgpu::{BindGroup, BindGroupLayout, Device, Sampler, Texture, TextureView};

use super::GpuTexture;

pub struct NodeAtlas {
    size: [u32; 3],
}

impl GpuTexture for NodeAtlas {
    fn bind(&self, device: &Device) -> (Texture, BindGroup, BindGroupLayout) {
        let texture = self.create_texture(device);
        let view = self.create_texture_view(&texture);
        let sampler = self.create_texture_sampler(device);
        let bind_group_layout = self.create_bind_group_layout(device);
        let bind_group = self.create_bind_group(device, &bind_group_layout, &view, &sampler);

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
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("Atlas Texture"),
            view_formats: &[],
        });

        texture
    }

    fn create_texture_view(&self, texture: &Texture) -> TextureView {
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Atlas Texture View"),
            format: Some(wgpu::TextureFormat::Rgba8Unorm),
            ..Default::default()
        });

        texture_view
    }

    fn create_texture_sampler(&self, device: &Device) -> Sampler {
        let texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        texture_sampler
    }

    fn create_bind_group_layout(&self, device: &Device) -> BindGroupLayout {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Atlas Texture Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D3,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        bind_group_layout
    }

    fn create_bind_group(
        &self,
        device: &Device,
        layout: &BindGroupLayout,
        view: &TextureView,
        sampler: &Sampler,
    ) -> BindGroup {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Atlas Texture Bind Group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        });

        bind_group
    }
}
