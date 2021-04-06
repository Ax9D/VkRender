use std::collections::{HashMap, HashSet};

use spirv_reflect::{types::ReflectFormat, ShaderModule};

use super::ShaderDataType;

#[derive(Debug)]
pub struct ShaderVariable {
    pub(crate) name: String,
    pub(crate) dataType: ShaderDataType,
    pub(crate) location: u32,
}

pub struct ReflectionData {
    inputs: HashMap<String, ShaderVariable>,
    outputs: HashMap<String, ShaderVariable>,
    samplers: HashSet<String>,
}
impl ReflectionData {
    pub fn inputs(&self) -> &HashMap<String, ShaderVariable> {
        &self.inputs
    }
    pub fn outputs(&self) -> &HashMap<String, ShaderVariable> {
        &self.outputs
    }
    pub fn samplers(&self) -> &HashSet<String> {
        &self.samplers
    }
}

pub fn reflectShader(spirvData: &[u8]) -> Result<ReflectionData, Box<dyn std::error::Error>> {
    let module = ShaderModule::load_u8_data(spirvData)?;
    let inputVars = module.enumerate_input_variables(None)?;
    let mut inputs = HashMap::new();

    for var in inputVars {
        let layout = ShaderVariable {
            name: var.name.to_owned(),
            dataType: var.format.into(),
            location: var.location,
        };

        log::info!("{:?}", var.format);

        inputs.insert(var.name, layout);
    }

    let outputVars = module.enumerate_input_variables(None)?;

    let mut outputs = HashMap::new();

    for var in outputVars {
        let layout = ShaderVariable {
            name: var.name.to_owned(),
            dataType: var.format.into(),
            location: var.location,
        };

        outputs.insert(var.name, layout);
    }

    //Remove gl defaults

    let bindings = module.enumerate_descriptor_bindings(None)?;
    let mut samplers = HashSet::new();

    for binding in bindings {
        samplers.insert(binding.name);
    }

    for input in &inputs {
        log::info!("{:?}", input);
    }

    for sampler in &samplers {
        log::info!("{:?}", sampler);
    }

    for output in &outputs {
        log::info!("{:?}", output);
    }
    Ok(ReflectionData {
        inputs,
        outputs,
        samplers,
    })
}
