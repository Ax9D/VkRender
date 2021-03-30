use std::{error::Error, sync::Arc};

use ash::{
    extensions::khr::Surface,
    version::DeviceV1_0,
    vk::{self, CompositeAlphaFlagsKHR, ImageUsageFlags, SharingMode},
};
use winit::window::Window;

use super::{device::SwapchainSupportDetails, Device, PhysicalDevice};
pub struct Swapchain {
    device: Arc<super::Device>,
    inner: vk::SwapchainKHR,
    loader: ash::extensions::khr::Swapchain,
    images: Vec<vk::Image>,
    imageViews: Vec<vk::ImageView>,
}
// pub struct SwapchainConfig {
//     width: u32,
//     height: u32,
//     hdr: bool
// }

impl Swapchain {
    fn choosePresentMode(swapchainSupportDetails: &SwapchainSupportDetails) -> vk::PresentModeKHR {
        let mailboxPresentMode = swapchainSupportDetails
            .presentModes()
            .iter()
            .find(|&&mode| mode == vk::PresentModeKHR::MAILBOX);

        if mailboxPresentMode.is_none() {
            vk::PresentModeKHR::FIFO
        } else {
            vk::PresentModeKHR::MAILBOX
        }
    }
    fn chooseSurfaceFormat(
        swapchainSupportDetails: &SwapchainSupportDetails,
    ) -> Result<ash::vk::SurfaceFormatKHR, Box<dyn Error>> {
        let hdrBGRAFormat = swapchainSupportDetails.formats().iter().find(|format| {
            format.format == vk::Format::B8G8R8A8_SRGB
                && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        });

        if hdrBGRAFormat.is_none() {
            return Err("Physical device doesn't support HDR framebuffers".into());
        }

        Ok(*hdrBGRAFormat.unwrap())
    }
    pub(super) fn create(
        instance: &ash::Instance,
        device: &Arc<Device>,
        pdevice: &PhysicalDevice,
        window: &Window,
        &surface: &vk::SurfaceKHR,
        surfaceLoader: &Surface,
    ) -> Result<Swapchain, Box<dyn Error>> {
        unsafe {
            let swapchainSupportDetails = pdevice.swapchainSupportDetails();

            let presentMode = Self::choosePresentMode(&swapchainSupportDetails);

            let surfaceFormat = Self::chooseSurfaceFormat(&swapchainSupportDetails)?;

            let swapExtent = Self::chooseSwapExtent(swapchainSupportDetails.capabilities(), window);

            let mut imageCount = swapchainSupportDetails.capabilities().min_image_count + 1;

            if swapchainSupportDetails.capabilities().max_image_count > 0
                && imageCount > swapchainSupportDetails.capabilities().max_image_count
            {
                imageCount = swapchainSupportDetails.capabilities().max_image_count;
            }

            let swapchainLoader =
                ash::extensions::khr::Swapchain::new(instance, device.rawDevice());

            let mut swapchainCreateInfo = vk::SwapchainCreateInfoKHR::builder()
                .surface(surface)
                .min_image_count(imageCount)
                .image_format(surfaceFormat.format)
                .image_array_layers(1)
                .image_usage(ImageUsageFlags::COLOR_ATTACHMENT)
                .composite_alpha(CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(presentMode)
                .image_extent(swapExtent)
                .pre_transform(swapchainSupportDetails.capabilities().current_transform)
                .clipped(true);

            let queueFamilyIndices = [
                pdevice.graphicsQueueIndex() as u32,
                pdevice.presentQueueIndex() as u32,
            ];

            swapchainCreateInfo = if pdevice.graphicsQueueIndex() != pdevice.presentQueueIndex() {
                swapchainCreateInfo
                    .image_sharing_mode(SharingMode::CONCURRENT)
                    .queue_family_indices(&queueFamilyIndices)
            } else {
                swapchainCreateInfo.image_sharing_mode(SharingMode::EXCLUSIVE)
            };

            let swapchain = swapchainLoader.create_swapchain(&swapchainCreateInfo, None)?;

            let images = swapchainLoader.get_swapchain_images(swapchain)?;

            let imageViews = Self::createImageViews(device, &images, surfaceFormat.format)?;

            Ok(Self {
                device: device.clone(),
                inner: swapchain,
                loader: swapchainLoader,
                images,
                imageViews,
            })
        }
    }
    fn createImageViews(
        device: &Arc<super::Device>,
        images: &Vec<vk::Image>,
        format: vk::Format,
    ) -> Result<Vec<vk::ImageView>, Box<dyn Error>> {
        let mut imageViews = Vec::new();

        for image in images {
            imageViews.push(super::ImageView::default(device, image, format)?);
        }
        Ok(imageViews)
    }
    fn chooseSwapExtent(
        capabilities: &vk::SurfaceCapabilitiesKHR,
        window: &Window,
    ) -> vk::Extent2D {
        log::info!("{:?}", window.inner_size());

        if capabilities.current_extent.width != u32::MAX {
            capabilities.current_extent
        } else {
            let physicalSize = window.inner_size();
            vk::Extent2D::builder()
                .width(physicalSize.width.clamp(
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ))
                .height(physicalSize.height.clamp(
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ))
                .build()
        }
    }
}
impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            self.imageViews
                .iter()
                .for_each(|&imageView| self.device.rawDevice().destroy_image_view(imageView, None));

            self.loader.destroy_swapchain(self.inner, None);
        }
    }
}
