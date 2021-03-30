use ash::vk;
pub struct Pipeline {

}

use std::error::Error;
impl Pipeline {
    pub fn create() -> Result<Pipeline, Box<dyn Error>> {
        let x = vk::PipelineVertexInputStateCreateInfo::builder();


        todo!()
    }
}