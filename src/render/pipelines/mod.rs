use wgpu::{ComputePipeline, PipelineLayout, RenderPipeline};

use super::WgpuContext;

mod solid;
pub use solid::*;

mod compute;
pub use compute::*;

pub trait Pipeline {
    fn get(&self, context: &WgpuContext, layout: PipelineLayout) -> RenderPipeline;
}

pub trait CPipeline {
    fn get(&self, context: &WgpuContext, layout: PipelineLayout) -> ComputePipeline;
}
