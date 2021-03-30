use std::{ffi::{OsStr, OsString}, path::{Path, PathBuf}, sync::{Arc, Mutex}};

use ash::{version::DeviceV1_0, vk};
enum ShaderFormat {
    Source(String),
    Spirv(Vec<u8>),
}
use c_str_macro::c_str;
use thiserror::Error;
#[derive(Error, Debug)]
pub enum ShaderCreateError {
    #[error("Couldn't recognise shader format for: {0:?}!\n .glsl -> GLSL shader\n .hlsl -> HLSL shader\n .spv -> SPIR-V blob")]
    UnrecognizedExtension(OsString),
    #[error("Failed to compile shader: {0:?}")]
    CompilationError(OsString)
}

impl ShaderFormat {
    
}
pub struct ShaderBuilder<'a> {
    device: Arc<super::Device>,
    vertexInfo: ShaderCompileInfo<'a>,
    fragmentInfo: ShaderCompileInfo<'a>,
}
pub struct ShaderInfo<'a> {
    name: &'a str,
    entryPoint: &'static str
}
pub struct ShaderCompileInfo<'a> {
    info: ShaderInfo<'a>,
    format: ShaderFormat
}
struct ShaderCompiler {
    inner: shaderc::Compiler
}
impl ShaderCompiler {
    fn new() -> ShaderCompiler {
        ShaderCompiler {
            inner: shaderc::Compiler::new().unwrap()
        }
    }
}
unsafe impl Send for ShaderCompiler{}

use std::ops::Deref;
impl Deref for ShaderCompiler {
    type Target = shaderc::Compiler;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
use std::ops::DerefMut;
impl DerefMut for ShaderCompiler {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
lazy_static! {
    static ref COMPILER: Mutex<ShaderCompiler> = Mutex::new(ShaderCompiler::new());
}
impl<'a> ShaderBuilder<'a> {
    pub fn VertexAndFragment(
        device: Arc<super::Device>,
        vertexInfo: ShaderCompileInfo<'a>,
        fragmentInfo: ShaderCompileInfo<'a>,
    ) -> ShaderBuilder<'a> {
        ShaderBuilder {
            device,
            vertexInfo,
            fragmentInfo,
        }
    }
    pub fn Tesselation(&mut self, info: ShaderCompileInfo<'a>) -> &mut Self {
        todo!()
    }
    
    pub fn build(self) -> Result<Arc<Shader>, Box<dyn std::error::Error>> {
        
        let vertexModule = Self::createShaderModule(&self.device, &self.vertexInfo, shaderc::ShaderKind::Vertex)?;
        let fragmentModule = Self::createShaderModule(&self.device,&self.fragmentInfo, shaderc::ShaderKind::Fragment)?;

        use std::ffi::CString;

        let pipelineStageCreateInfos = vec![ 

        vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::VERTEX)
        .module(vertexModule)
        .name(CString::new( self.vertexInfo.info.entryPoint ).unwrap().as_c_str())
        .build(),

        vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::FRAGMENT)
        .module(fragmentModule)
        .name(CString::new( self.fragmentInfo.info.entryPoint ).unwrap().as_c_str())
        .build()

        ];


        Ok (
            Arc::new(
            Shader{
                device: self.device.clone(),
                pipelineStageCreateInfos
            }
        )
        )
    }
    fn createShaderModule(device: &Arc<super::Device>, compileInfo: &ShaderCompileInfo, kind: shaderc::ShaderKind) -> Result<vk::ShaderModule, Box<dyn std::error::Error>> {
        use shaderc::*;
        let mut buf;
        let byteCode= match &compileInfo.format {
            ShaderFormat::Source(text) => {
                buf = Self::compileShader(&compileInfo.info, &text, ShaderKind::Vertex)?;
                &buf
            }
            ShaderFormat::Spirv(bytes) => {
                bytes
            }
        };
        
        let byteCode = unsafe {
            std::slice::from_raw_parts::<u32>(byteCode.as_ptr() as *const u32, byteCode.len() * std::mem::size_of::<u32>() / std::mem::size_of::<u8>())
        };

        let shaderCreateInfo = vk::ShaderModuleCreateInfo::builder()
        .code(byteCode)
        .build();

        Ok(
        unsafe {
            device.rawDevice().create_shader_module(&shaderCreateInfo, None)?
        }
    )
    }
    fn compileShader(shaderInfo: &ShaderInfo, text: &String,  kind: shaderc::ShaderKind) -> Result<Vec<u8>, Box<dyn std::error::Error>>{
        let mut compiler = COMPILER.lock().unwrap();

        let artifact= compiler.compile_into_spirv(text.as_str(), shaderc::ShaderKind::Vertex, shaderInfo.name, shaderInfo.entryPoint, None)?;
        log::warn!("{} Compilation Warning: {}",shaderInfo.name, artifact.get_warning_messages());

        Ok (
            artifact.as_binary_u8().to_owned()
        )
    }
}

pub struct Shader {
    device: Arc<super::Device>,
    pipelineStageCreateInfos: Vec<vk::PipelineShaderStageCreateInfo>,
}

impl Shader {
    pub (crate) fn pipelineStageCreateInfos(&self) -> &Vec<vk::PipelineShaderStageCreateInfo> {
        &self.pipelineStageCreateInfos
    }
}
