use wgpu::{BindGroup, BindGroupLayout, Buffer, Device, VertexBufferLayout};

pub trait GpuUniform {
    fn bind(&self, device: &Device) -> (Buffer, Vec<u8>, BindGroup, BindGroupLayout);
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
