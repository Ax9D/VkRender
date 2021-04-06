pub mod ImageView {
    use std::{error::Error, sync::Arc};

    pub use super::super::Device;
    use ash::version::DeviceV1_0;
    pub use ash::vk::{self};

    pub fn default(
        device: &Arc<Device>,
        &image: &vk::Image,
        format: vk::Format,
    ) -> Result<vk::ImageView, Box<dyn Error>> {
        let RGBASWIZZLE = {
            vk::ComponentMapping::builder()
                .r(vk::ComponentSwizzle::IDENTITY)
                .g(vk::ComponentSwizzle::IDENTITY)
                .b(vk::ComponentSwizzle::IDENTITY)
                .a(vk::ComponentSwizzle::IDENTITY)
                .build()
        };

        let subresourceRange = vk::ImageSubresourceRange::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1)
            .build();

        let createInfo = vk::ImageViewCreateInfo::builder()
            .image(image)
            .format(format)
            .view_type(vk::ImageViewType::TYPE_2D)
            .components(RGBASWIZZLE)
            .subresource_range(subresourceRange);

        let imageView = unsafe { device.raw().create_image_view(&createInfo, None)? };

        Ok(imageView)
    }
}

pub mod Image {
    
}