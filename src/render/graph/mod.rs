//pub mod renderpass;
mod node;
mod pipeline;
mod renderpass;

use ash::vk;
pub use pipeline::PipelineCreateInfo;
//pub use renderpass::RenderpassBuilder;

pub struct Graph {
    images: Vec<vk::Image>,
    imageViews: Vec<vk::ImageView>,
    allocations: Vec<gpu_allocator::SubAllocation>,
    framebuffers: Vec<vk::Framebuffer>,
}
struct CompiledData {
    
}

pub struct GraphBuilder {}

impl GraphBuilder {
    pub fn new() -> Self {
        Self {}
    }
    pub fn addRenderpass(&mut self) -> &mut Self {
        todo!()
    }
}
