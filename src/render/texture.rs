use ash::vk;


pub enum ColorFormat {
    RGBA8,
    RGBA32F,
}
impl Into<vk::Format> for ColorFormat {
    fn into(self) -> vk::Format {
        match self {
            ColorFormat::RGBA8 => vk::Format::R8G8B8_UINT,
            ColorFormat::RGBA32F => vk::Format::R32G32B32_SFLOAT,
        }
    }
}
impl From<vk::Format> for ColorFormat {
    fn from(x: vk::Format) -> Self {
        match x {
            vk::Format::R8G8B8_UINT => ColorFormat::RGBA8,
            vk::Format::R32G32B32_SFLOAT => ColorFormat::RGBA32F,
            _=>{todo!()}
        } 
    }
}
pub enum DepthStencilFormat {
    Depth16,
    Depth24Stencil8,
}
impl Into<vk::Format> for DepthStencilFormat {
    fn into(self) -> vk::Format {
        match self {
            DepthStencilFormat::Depth16 => vk::Format::D16_UNORM,
            DepthStencilFormat::Depth24Stencil8 => vk::Format::D24_UNORM_S8_UINT,
        }
    }
}
impl From<vk::Format> for DepthStencilFormat {
 fn from(x: vk::Format) -> Self {
     match x {
        vk::Format::D16_UNORM => DepthStencilFormat::Depth16,
        vk::Format::D24_UNORM_S8_UINT => DepthStencilFormat::Depth24Stencil8,
        _=> todo!()
     }
 }   
}