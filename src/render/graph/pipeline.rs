use ash::{
    version::DeviceV1_0,
    vk::{self, ColorComponentFlags, PipelineInputAssemblyStateCreateInfo, SampleCountFlags},
};

use std::{cmp::max, error::Error, ffi::CString, sync::Arc};

use crate::Shader;

pub enum BlendMode {
    None,
}
// pub struct PipelineDescriptor {
//     scissor: (f32, f32, f32, f32),
//     msaa: bool,
// }
pub enum PrimitiveTopology {
    Point,
    Triangle,
    Line,
}
impl Default for PrimitiveTopology {
    fn default() -> Self {
        Self::Triangle
    }
}

pub struct PipelineCreateInfo {
    pub(crate) shaderStageCreateInfos: Vec<vk::PipelineShaderStageCreateInfo>,
    pub(crate) inputAssemblyCreateInfo: vk::PipelineInputAssemblyStateCreateInfo,
    pub(crate) viewportStateCreateInfo: vk::PipelineViewportStateCreateInfo,
    pub(crate) rasterizationStateCreateInfo: vk::PipelineRasterizationStateCreateInfo,
    pub(crate) multisampleStateCreateInfo: vk::PipelineMultisampleStateCreateInfo,
    pub(crate) colorBlendStateCreateInfo: vk::PipelineColorBlendStateCreateInfo,
    pub(crate) dynamicStateCreateInfo: vk::PipelineDynamicStateCreateInfo,
    pub(crate) vertexInputStateCreateInfo: vk::PipelineVertexInputStateCreateInfo,
    pub(crate) shader: Arc<crate::Shader>,
}

use thiserror::Error;
#[derive(Error, Debug)]
pub enum PipelineCreateError {
    #[error("No shader provided for pipeline creation!")]
    NoShader,
}
pub struct VertexInputLayout {}
pub struct PipelineDesciptor {
    topology: PrimitiveTopology,
    msaa: bool,
    shader: Arc<Shader>,
    vertexInputLayout: VertexInputLayout,
    //blend
}

pub struct Pipeline {
    device: Arc<crate::Device>,
    createInfo: PipelineCreateInfo,
    pipeline: vk::Pipeline,
    layout: vk::PipelineLayout,
}
impl Drop for Pipeline {
    fn drop(&mut self) {
        unsafe {
            self.device.raw().destroy_pipeline_layout(self.layout, None);
            self.device.raw().destroy_pipeline(self.pipeline, None);
        }
    }
}
impl PipelineCreateInfo {
    pub(crate) fn new(descriptor: PipelineDesciptor) -> Self {
        let topology = match descriptor.topology {
            PrimitiveTopology::Point => vk::PrimitiveTopology::POINT_LIST,
            PrimitiveTopology::Triangle => vk::PrimitiveTopology::TRIANGLE_LIST,
            PrimitiveTopology::Line => vk::PrimitiveTopology::LINE_LIST,
        };

        let vertexInputStateCreateInfo = vk::PipelineVertexInputStateCreateInfo::builder().build();

        let inputAssemblyCreateInfo = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .primitive_restart_enable(false)
            .topology(topology)
            .build();

        let viewport = vk::Viewport::builder().build();

        let scissor = vk::Rect2D::builder()
            // .offset(vk::Offset2D::default())
            // .extent(vk::Extent2D::builder().width(width).height(height).build())
            .build();

        let viewportStateCreateInfo = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(&[viewport])
            .scissors(&[scissor])
            .build();

        let rasterizationStateCreateInfo = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .polygon_mode(vk::PolygonMode::FILL)
            .front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false)
            .build();

        let multisampleStateCreateInfo = vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(descriptor.msaa)
            .rasterization_samples(SampleCountFlags::TYPE_1)
            .build();

        let attachment = vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(ColorComponentFlags::all())
            .blend_enable(false)
            .build();

        let colorBlendStateCreateInfo = vk::PipelineColorBlendStateCreateInfo::builder()
            .attachments(&[attachment])
            .build();

        let pipelineLayout = vk::PipelineLayoutCreateInfo::builder().build();

        //let pipelineLayout = unsafe {device.rawDevice().create_pipeline_layout(&&pipelineLayout, None).unwrap()};

        let shaderStageCreateInfos = vec![
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(descriptor.shader.vertex.module)
                .name(
                    CString::new(descriptor.shader.vertex.compileInfo.entryPoint.as_str())
                        .unwrap()
                        .as_c_str(),
                )
                .build(),
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(descriptor.shader.fragment.module)
                .name(
                    CString::new(descriptor.shader.fragment.compileInfo.entryPoint.as_str())
                        .unwrap()
                        .as_c_str(),
                )
                .build(),
        ];

        let dynamicStateCreateInfo = vk::PipelineDynamicStateCreateInfo::builder()
            .dynamic_states(&[
                vk::DynamicState::VIEWPORT,
                vk::DynamicState::LINE_WIDTH,
                vk::DynamicState::SCISSOR,
            ])
            .build();

        Self {
            vertexInputStateCreateInfo,
            inputAssemblyCreateInfo,
            viewportStateCreateInfo,
            rasterizationStateCreateInfo,
            multisampleStateCreateInfo,
            colorBlendStateCreateInfo,

            shaderStageCreateInfos,
            dynamicStateCreateInfo,
            shader: descriptor.shader,
        }
    }
    pub(crate) fn create(
        self,
        device: &Arc<crate::Device>,
        renderpass: vk::RenderPass,
    ) -> Result<Pipeline, Box<dyn std::error::Error>> {
        let layoutCreateInfo = vk::PipelineLayoutCreateInfo::builder().build();

        let pipelineLayout = unsafe {
            device
                .raw()
                .create_pipeline_layout(&layoutCreateInfo, None)?
        };

        let graphicsPipelineCreateInfo = vk::GraphicsPipelineCreateInfo::builder()
            .vertex_input_state(&self.vertexInputStateCreateInfo)
            .input_assembly_state(&self.inputAssemblyCreateInfo)
            .viewport_state(&self.viewportStateCreateInfo)
            .rasterization_state(&self.rasterizationStateCreateInfo)
            .multisample_state(&self.multisampleStateCreateInfo)
            .dynamic_state(&self.dynamicStateCreateInfo)
            .color_blend_state(&self.colorBlendStateCreateInfo)
            .layout(pipelineLayout)
            .render_pass(renderpass)
            .subpass(0)
            .build();

        let pipeline = unsafe {
            device
                .raw()
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &[graphicsPipelineCreateInfo],
                    None,
                )
                .unwrap()
        };

        Ok(Pipeline {
            device: device.clone(),
            createInfo: self,
            pipeline: pipeline[0],
            layout: pipelineLayout,
        })
    }
    // pub fn build(self) -> Result<Pipeline, Box<dyn Error>> {
    //     if self.shaderStageCreateInfos.is_none() {
    //         return Err (Box::new(PipelineCreateError::NoShader))
    //     }
    //     todo!()
    // }
}
impl PipelineCreateInfo {
    // pub fn create() -> Result<Pipeline, Box<dyn Error>> {
    //     let x = vk::PipelineVertexInputStateCreateInfo::builder();

    //     todo!()
    // }
}
