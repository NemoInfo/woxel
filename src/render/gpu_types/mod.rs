use wgpu::{BindGroup, BindGroupLayout, Buffer, Device, Texture, VertexBufferLayout};

pub trait GpuUniform {
    fn bind(&self, device: &Device) -> (Buffer, Vec<u8>, BindGroup, BindGroupLayout);
}

pub trait GpuTexture {
    fn bind(&self, device: &Device) -> (Texture, BindGroup, BindGroupLayout);
}

pub trait GpuPrimitive {
    fn data(&self) -> Vec<u8>;
    fn desc<'a>() -> VertexBufferLayout<'a>;
}

mod camera;
pub use camera::CameraUniform;

mod vertex;
pub use vertex::GpuVertex;

mod quad;
pub use quad::{GpuQuad, GPU_QUAD};

mod ray;
pub use ray::RayUniform;

mod state;
pub use state::StateUniform;

mod texture;
pub use texture::*;

mod atlas;
pub use atlas::*;
