use wgpu::{PipelineLayout, RenderPipeline};

use super::WgpuContext;

mod solid;
pub use solid::*;

pub trait Pipeline {
    fn get(&self, context: &WgpuContext, layout: PipelineLayout) -> RenderPipeline;
}
