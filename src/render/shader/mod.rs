use std::{
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use ash::{version::DeviceV1_0, vk};

mod reflection;

use c_str_macro::c_str;
use thiserror::Error;
#[derive(Error, Debug)]
pub enum ShaderCreateError {
    #[error("Couldn't recognise shader format for: {0:?}!\n .glsl -> GLSL shader\n .hlsl -> HLSL shader\n .spv -> SPIR-V blob")]
    UnrecognizedExtension(OsString),
    #[error("Failed to compile shader: {0:?}")]
    CompilationError(OsString),
}
pub enum ShaderData {
    Source(String),
    Spirv(Vec<u8>),
}

#[derive(Debug)]
pub enum ShaderDataType {
    Int,
    UInt,
    Float,
    Vec2f,
    Vec3f,
    Vec4f,
}
use spirv_reflect::types::ReflectFormat;

impl From<ReflectFormat> for ShaderDataType {
    fn from(x: ReflectFormat) -> Self {
        match x {
            ReflectFormat::R32_SFLOAT => Self::Float,
            ReflectFormat::R32G32B32_SFLOAT => Self::Vec3f,
            ReflectFormat::R32G32B32A32_SFLOAT => Self::Vec4f,
            ReflectFormat::R32_SINT => Self::Int,
            ReflectFormat::R32_UINT => Self::UInt,
            _ => todo!(),
        }
    }
}

pub struct ShaderCompileInfo {
    pub(crate) entryPoint: String,
    pub(crate) data: ShaderData,
}
struct ShaderCompiler {
    inner: shaderc::Compiler,
}
impl ShaderCompiler {
    fn new() -> ShaderCompiler {
        ShaderCompiler {
            inner: shaderc::Compiler::new().unwrap(),
        }
    }
}
unsafe impl Send for ShaderCompiler {}

use std::ops::Deref;
impl Deref for ShaderCompiler {
    type Target = shaderc::Compiler;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
use std::ops::DerefMut;

use self::reflection::{reflectShader, ReflectionData};
impl DerefMut for ShaderCompiler {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
lazy_static! {
    static ref COMPILER: Mutex<ShaderCompiler> = Mutex::new(ShaderCompiler::new());
}
fn processShader(
    name: &String,
    compileInfo: &ShaderCompileInfo,
    kind: shaderc::ShaderKind,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    use shaderc::*;
    let byteCode = match &compileInfo.data {
        ShaderData::Source(text) => {
            compileShader(&name, &compileInfo.entryPoint, &text, kind)?
        }
        ShaderData::Spirv(bytes) => bytes.clone(),
    };

    Ok(byteCode)
}
fn createShaderModule(
    device: &Arc<super::Device>,
    byteCode: &[u8],
) -> Result<vk::ShaderModule, Box<dyn std::error::Error>> {
    let byteCode = unsafe {
        std::slice::from_raw_parts::<u32>(
            byteCode.as_ptr() as *const u32,
            byteCode.len() / (std::mem::size_of::<u32>() / std::mem::size_of::<u8>()),
        )
    };

    let shaderCreateInfo = vk::ShaderModuleCreateInfo::builder().code(byteCode).build();

    Ok(unsafe { device.raw().create_shader_module(&shaderCreateInfo, None)? })
}
fn compileShader(
    name: &str,
    entryPoint: &str,
    text: &String,
    kind: shaderc::ShaderKind,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut compiler = COMPILER.lock().unwrap();

    let artifact = compiler.compile_into_spirv(text.as_str(), kind, name, entryPoint, None)?;

    if artifact.get_num_warnings() > 0 {
        log::warn!(
            "{}[{:?} Shader] Compilation Warning: {}",
            name,
            kind,
            artifact.get_warning_messages()
        );
    }

    Ok(artifact.as_binary_u8().to_owned())
}
pub struct ShaderInfo {
    pub(crate) module: vk::ShaderModule,
    pub(crate) compileInfo: ShaderCompileInfo,
    pub(crate) reflectionData: ReflectionData,
}
pub struct Shader {
    device: Arc<super::Device>,
    pub(crate) name: String,
    pub(crate) vertex: ShaderInfo,
    pub(crate) fragment: ShaderInfo,
}

impl Shader {
    pub fn create(
        device: &Arc<super::Device>,
        name: String, 
        vertexInfo: ShaderCompileInfo,
        fragmentInfo: ShaderCompileInfo,
    ) -> Result<Arc<Self>, Box<dyn std::error::Error>> {
        let vertexBytes = processShader(&(name.to_owned() + "_vertex"), &vertexInfo, shaderc::ShaderKind::Vertex)?;
        let fragmentBytes = processShader(&(name.to_owned() +"_fragment"),&fragmentInfo, shaderc::ShaderKind::Fragment)?;

        let vertexReflect = reflectShader(&vertexBytes)?;
        let fragmentReflect = reflectShader(&&fragmentBytes)?;

        let vertexModule = createShaderModule(&device, &vertexBytes)?;
        let fragmentModule = createShaderModule(&device, &fragmentBytes)?;

        let vertex = ShaderInfo {
            module: vertexModule,
            compileInfo: vertexInfo,
            reflectionData: vertexReflect,
        };

        let fragment = ShaderInfo {
            module: fragmentModule,
            compileInfo: fragmentInfo,
            reflectionData: fragmentReflect,
        };
        // let pipelineStageCreateInfos = vec![

        // vk::PipelineShaderStageCreateInfo::builder()
        // .stage(vk::ShaderStageFlags::VERTEX)
        // .module(vertexModule)
        // .name(CString::new( vertexInfo.info.entryPoint ).unwrap().as_c_str())
        // .build(),

        // vk::PipelineShaderStageCreateInfo::builder()
        // .stage(vk::ShaderStageFlags::FRAGMENT)
        // .module(fragmentModule)
        // .name(CString::new( fragmentInfo.info.entryPoint ).unwrap().as_c_str())
        // .build()

        //];

        Ok(Arc::new(Shader {
            name,
            device: device.clone(),
            vertex,
            fragment,
        }))
    }
}
