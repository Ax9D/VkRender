use std::sync::Arc;

use ash::{version::DeviceV1_0, vk};
use gpu_allocator::SubAllocation;

use super::{ColorFormat, DepthStencilFormat};

pub trait RenderTarget {}
pub struct Framebuffer{
    device: Arc<crate::Device>,

    inner: vk::Framebuffer,
    images: Vec<vk::Image>,
    imageViews: Vec<vk::ImageView>,
    samplers: Vec<vk::Sampler>,
    allocations: Vec<SubAllocation>,

    colorAttachments: Vec<vk::Format>,
    depthAttachments: Vec<vk::Format>,
    renderpass: vk::RenderPass,
}
impl Framebuffer {
    pub fn create(device: &Arc<crate::Device>, renderpass: vk::RenderPass, colorAttachments: &[vk::Format], depthAttachments: &[vk::Format], width: u32, height: u32) -> Result<Self, Box<dyn std::error::Error>> {
        let mut allocations = Vec::new();
        let mut images = Vec::new();
        let mut imageViews= Vec::new();
        let mut samplers= Vec::new();

        for attachment in colorAttachments {
            let imageCreateInfo = 
            vk::ImageCreateInfo::builder()
            .format(*attachment)
            .image_type(vk::ImageType::TYPE_2D)
            .extent(vk::Extent3D::builder().width(width).height(height).depth(1).build())
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED)
            .build();

            let image = unsafe {device.raw().create_image(&imageCreateInfo, None)?};

            let requirements = unsafe { device.raw().get_image_memory_requirements(image) };
            
            use gpu_allocator::*;
            let alloc = device.allocateDeviceMemory(AllocationCreateDesc{
                name: "ColorAttachment",
                requirements,
                location: MemoryLocation::GpuOnly,
                linear: false,
            })?;

            unsafe {device.raw().bind_image_memory(image, alloc.memory(), alloc.offset())?};
            

            let imageViewCreateInfo = 
            vk::ImageViewCreateInfo::builder()
            .format(*attachment)
            .subresource_range(vk::ImageSubresourceRange::builder().base_mip_level(0).level_count(1).base_array_layer(1).layer_count(1).build())
            .image(image)
            ;

            let imageView = 
            unsafe {device.raw().create_image_view(&imageViewCreateInfo, None)}?;
            
            let samplerCreateInfo = 
            vk::SamplerCreateInfo::builder()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .mip_lod_bias(0.0)
            .max_anisotropy(1.0)
            .min_lod(0.0)
            .max_lod(1.0)
            .build();

            let sampler = unsafe {device.raw().create_sampler(&samplerCreateInfo, None)}?;


            allocations.push(alloc);
            images.push(image);
            imageViews.push(imageView);
            samplers.push(sampler);
        } 

        for attachment in depthAttachments {
            let imageCreateInfo = 
            vk::ImageCreateInfo::builder()
            .format(*attachment)
            .image_type(vk::ImageType::TYPE_2D)
            .extent(vk::Extent3D::builder().width(width).height(height).depth(1).build())
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | vk::ImageUsageFlags::SAMPLED)
            .build();

            let image = unsafe {device.raw().create_image(&imageCreateInfo, None)?};

            let requirements = unsafe { device.raw().get_image_memory_requirements(image) };
            
            use gpu_allocator::*;
            let alloc = device.allocateDeviceMemory(AllocationCreateDesc{
                name: "DepthAttachment",
                requirements,
                location: MemoryLocation::GpuOnly,
                linear: false,
            })?;

            unsafe {device.raw().bind_image_memory(image, alloc.memory(), alloc.offset())?};
            

            let imageViewCreateInfo = 
            vk::ImageViewCreateInfo::builder()
            .format(*attachment)
            .subresource_range(vk::ImageSubresourceRange::builder()
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(1)
            .layer_count(1)
            .aspect_mask(vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL)
            .build())
            .image(image)
            ;

            let imageView = 
            unsafe {device.raw().create_image_view(&imageViewCreateInfo, None)}?;
            
            let samplerCreateInfo = 
            vk::SamplerCreateInfo::builder()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .mip_lod_bias(0.0)
            .max_anisotropy(1.0)
            .min_lod(0.0)
            .max_lod(1.0)
            .build();

            let sampler = unsafe {device.raw().create_sampler(&samplerCreateInfo, None)}?;


            allocations.push(alloc);
            images.push(image);
            imageViews.push(imageView);
            samplers.push(sampler);
        }  

        let framebufferCreateInfo = 
        vk::FramebufferCreateInfo::builder()
        .attachments(&imageViews)
        .width(width)
        .height(height)
        .layers(1)
        .build();

        let inner = unsafe {device.raw().create_framebuffer(&framebufferCreateInfo, None)}?;

        let colorAttachments = colorAttachments.to_vec();
        let depthAttachments = depthAttachments.to_vec();
        Ok(
        Self {
            inner, 
            images,
            imageViews,
            allocations,
            device: device.clone(),
            samplers,
            colorAttachments,
            depthAttachments,
            renderpass
        }
    )
    }

    pub fn recreate(self, width: u32, height: u32) -> Result<Self, Box<dyn std::error::Error>>{
        Self::create(&self.device, self.renderpass, &self.colorAttachments, &self.depthAttachments, width, height)
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
        self.device.raw().destroy_framebuffer(self.inner, None);
        
        for &sampler in &self.samplers {
            self.device.raw().destroy_sampler(sampler, None);
        }
        for &view in &self.imageViews {
            self.device.raw().destroy_image_view(view, None);
        }
        for &image in &self.images {
            self.device.raw().destroy_image(image, None);
        }
        for allocation in &self.allocations {
            self.device.freeDeviceMemory(allocation.clone()).unwrap();
        }
    }
    }
}