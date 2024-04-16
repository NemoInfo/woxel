use wgpu::{BindGroup, BindGroupLayout, Device, Texture, TextureView};

pub struct NodeAtlas {
    size5: [u32; 3],
    size4: [u32; 3],
    size3: [u32; 3],
}

impl NodeAtlas {
    pub fn new(size5: [u32; 3], size4: [u32; 3], size3: [u32; 3]) -> Self {
        Self {
            size5,
            size4,
            size3,
        }
    }

    pub fn bind(&self, device: &Device) -> ([Texture; 3], BindGroup, BindGroupLayout) {
        let textures = self.create_textures(device);
        let views = textures
            .iter()
            .map(|texture| self.create_texture_view(&texture))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let bind_group_layout = self.create_bind_group_layout(device);
        let bind_group = self.create_bind_group(device, &bind_group_layout, &views);

        (textures, bind_group, bind_group_layout)
    }

    fn create_textures(&self, device: &Device) -> [Texture; 3] {
<<<<<<< HEAD
        let texture5 = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: self.size5[0],
                height: self.size5[1],
                depth_or_array_layers: self.size5[2],
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D3,
            format: wgpu::TextureFormat::R32Uint,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("Atlas Texture"),
            view_formats: &[],
        });

        let texture4 = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: self.size4[0],
                height: self.size4[1],
                depth_or_array_layers: self.size4[2],
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D3,
            format: wgpu::TextureFormat::R32Uint,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("Atlas Texture"),
            view_formats: &[],
        });

        let texture3 = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: self.size3[0],
                height: self.size3[1],
                depth_or_array_layers: self.size3[2],
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D3,
            format: wgpu::TextureFormat::R32Uint,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("Atlas Texture"),
            view_formats: &[],
        });

        [texture5, texture4, texture3]
=======
        [self.size5, self.size4, self.size3].map(|[w, h, d]| {
            device.create_texture(&wgpu::TextureDescriptor {
                size: wgpu::Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: d,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D3,
                format: wgpu::TextureFormat::R32Uint,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                label: Some("Atlas Texture"),
                view_formats: &[],
            })
        })
>>>>>>> main
    }

    fn create_texture_view(&self, texture: &Texture) -> TextureView {
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Atlas Texture View"),
            format: Some(wgpu::TextureFormat::R32Uint),
            ..Default::default()
        });

        texture_view
    }

    fn create_bind_group_layout(&self, device: &Device) -> BindGroupLayout {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Atlas Texture Bind Group Layout"),
<<<<<<< HEAD
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Uint,
                        view_dimension: wgpu::TextureViewDimension::D3,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Uint,
                        view_dimension: wgpu::TextureViewDimension::D3,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Uint,
                        view_dimension: wgpu::TextureViewDimension::D3,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
=======
            entries: &[0, 1, 2].map(|binding| wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Uint,
                    view_dimension: wgpu::TextureViewDimension::D3,
                    multisampled: false,
                },
                count: None,
            }),
>>>>>>> main
        });

        bind_group_layout
    }

    fn create_bind_group(
        &self,
        device: &Device,
        layout: &BindGroupLayout,
<<<<<<< HEAD
        [view5, view4, view3]: &[TextureView; 3],
=======
        views: &[TextureView; 3],
>>>>>>> main
    ) -> BindGroup {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Atlas Texture Bind Group"),
            layout,
<<<<<<< HEAD
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(view5),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(view4),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(view3),
                },
            ],
=======
            entries: &[0, 1, 2].map(|binding| wgpu::BindGroupEntry {
                binding,
                resource: wgpu::BindingResource::TextureView(&views[binding as usize]),
            }),
>>>>>>> main
        });

        bind_group
    }
}
