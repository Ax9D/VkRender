use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use ash::{version::DeviceV1_0, vk};

use crate::render::{ColorFormat, DepthStencilFormat};

use super::PipelineCreateInfo;

pub trait DrawState {}
pub struct ColorInput {
    name: &'static str,
    uniformName: &'static str,
}
pub struct DepthStencilInput {
    name: &'static str,
    uniformName: &'static str,
}
pub struct ColorOutput {
    name: &'static str,
    layoutName: &'static str,
    format: ColorFormat,
}
pub struct DepthStencilOutput {
    name: &'static str,
    format: DepthStencilFormat,
}

use thiserror::Error;
#[derive(Error, Debug)]
pub enum RenderpassValidationError {
    #[error("In {0:?}: cyclic dependency detected!")]
    CyclicDependency(&'static str),

    #[error("In pass {0:?}, Uniform {1:?} corresponding to color input {2:?}  was not found in fragment shader of: {3:?}!")]
    ShaderUniformNotFound(&'static str, &'static str, &'static str, String),
    #[error("In pass {0:?}, Shader output {1:?} corresponding to color output {2:?} was not found in fragment shader of: {3:?}!")]
    ShaderOutputNotFound(&'static str, &'static str, &'static str, String),
}

pub struct RenderpassBuilder {
    name: &'static str,
    pipeline: PipelineCreateInfo,
    drawState: Box<dyn DrawState>,

    colorInputs: HashMap<&'static str, ColorInput>,
    colorOutputs: HashMap<&'static str, ColorOutput>,

    depthInput: Option<DepthStencilInput>,
    depthOutput: Option<DepthStencilOutput>,
}

impl RenderpassBuilder {
    pub fn new<T: 'static + DrawState>(
        name: &'static str,
        pipeline: PipelineCreateInfo,
        drawState: T,
    ) -> Self {
        let drawState: Box<dyn DrawState> = Box::new(drawState);

        let depthInput = None;
        let depthOutput = None;

        let colorInputs = HashMap::new();
        let colorOutputs = HashMap::new();

        Self {
            name,
            pipeline,
            drawState,
            depthInput,
            depthOutput,
            colorInputs,
            colorOutputs,
        }
    }
    pub fn colorInput(&mut self, name: &'static str, uniformName: &'static str) -> &mut Self {
        self.colorInputs
            .insert(name, ColorInput { name, uniformName });

        self
    }

    pub fn colorOutput(
        &mut self,
        name: &'static str,
        layoutName: &'static str,
        format: ColorFormat,
    ) -> &mut Self {
        self.colorOutputs.insert(
            name,
            ColorOutput {
                name,
                layoutName,
                format,
            },
        );

        self
    }
    pub fn depthStencilInput(&mut self, name: &'static str, layoutName: &'static str) -> &mut Self {
        self.depthInput.replace(DepthStencilInput {
            name,
            uniformName: layoutName,
        });

        self
    }

    pub fn depthStencilOutput(
        &mut self,
        name: &'static str,
        layoutName: &'static str,
        format: DepthStencilFormat,
    ) -> &mut Self {
        self.depthOutput
            .replace(DepthStencilOutput { name, format });

        self
    }
    fn checkCyclicDeps(self) -> Result<Self, RenderpassValidationError> {
        let mut names = HashSet::new();

        let mut iCount = self.colorInputs.keys().len();
        let mut oCount = self.colorOutputs.keys().len();

        if let Some(depthInput) = &self.depthInput {
            iCount += 1;
            names.insert(depthInput.name);
        }

        if let Some(depthOutput) = &self.depthOutput {
            oCount += 1;
            names.insert(depthOutput.name);
        }
        for (&input, _) in &self.colorInputs {
            names.insert(input);
        }

        for (&output, _) in &self.colorOutputs {
            names.insert(output);
        }

        if names.len() < iCount + oCount {
            return Err(RenderpassValidationError::CyclicDependency(self.name));
        }

        Ok(self)
    }
    fn checkUniformValidity(self) -> Result<Self, RenderpassValidationError> {
        let fragment = &self.pipeline.shader.fragment;

        for (&name, colorInput) in &self.colorInputs {
            if fragment
                .reflectionData
                .samplers()
                .get(colorInput.uniformName)
                .is_none()
            {
                return Err(RenderpassValidationError::ShaderUniformNotFound(
                    self.name,
                    colorInput.uniformName,
                    name,
                    self.pipeline.shader.name.to_owned(),
                ));
            }
        }

        for (&name, colorOutput) in &self.colorOutputs {
            if fragment
                .reflectionData
                .outputs()
                .get(colorOutput.layoutName)
                .is_none()
            {
                return Err(RenderpassValidationError::ShaderOutputNotFound(
                    self.name,
                    colorOutput.layoutName,
                    name,
                    self.pipeline.shader.name.to_owned(),
                ));
            }
        }

        if let Some(depthInput) = &self.depthInput {
            if fragment
                .reflectionData
                .samplers()
                .get(depthInput.uniformName)
                .is_none()
            {
                return Err(RenderpassValidationError::ShaderUniformNotFound(
                    self.name,
                    depthInput.uniformName,
                    depthInput.name,
                    self.pipeline.shader.name.to_owned(),
                ));
            }
        }

        Ok(self)
    }
    fn validate(self) -> Result<Self, RenderpassValidationError> {
        Ok(self.checkUniformValidity()?.checkCyclicDeps()?)
    }
    pub fn build(self) -> Result<Renderpass, Box<dyn std::error::Error>> {
        Ok(
            Renderpass {
                data: self.validate()?,
                framebuffer: vk::Framebuffer::null()
            }
        )
    }
}
pub struct Renderpass {
    data: RenderpassBuilder,
    framebuffer: vk::Framebuffer,
}