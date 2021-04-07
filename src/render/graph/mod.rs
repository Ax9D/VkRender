//pub mod renderpass;
mod node;
mod pipeline;
mod renderpass;
mod compiler;

use ash::vk;
pub use pipeline::PipelineCreateInfo;

use self::renderpass::Renderpass;
//pub use renderpass::RenderpassBuilder;

pub struct Graph {
    images: Vec<vk::Image>,
    imageViews: Vec<vk::ImageView>,
    allocations: Vec<gpu_allocator::SubAllocation>,
    framebuffers: Vec<vk::Framebuffer>,

}
impl Graph {
    pub (super) fn compile(passes: Vec<Renderpass>) -> Self{
        todo!()
    }
}
struct CompiledData {
    
}

pub struct GraphBuilder {
    passes: Vec<Renderpass>
}

impl GraphBuilder {
    pub fn new() -> Self {
        Self {
            passes: Vec::new()
        }
    }
    pub fn addRenderpass(&mut self, renderpass: Renderpass) -> &mut Self {
        self.passes.push(renderpass);

        self
    }
    pub fn build(self) -> Graph {
        Graph::compile(self.passes)
    }
}
