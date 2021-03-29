use std::{collections::HashSet, error::Error, ffi::CStr, ptr::swap};

use ash::{
    extensions::khr::{Surface, Swapchain},
    version::{DeviceV1_0, InstanceV1_0},
    vk::{self, GraphicsShaderGroupCreateInfoNV},
    Instance,
};

use super::graphics;

pub struct PhysicalDeviceInfo {
    device: vk::PhysicalDevice,
    graphicsQueue: Option<usize>,
    presentQueue: Option<usize>,
    swapchainSupportDetails: SwapchainSupportDetails,
    properties: ash::vk::PhysicalDeviceProperties,
    suitable: bool,
}
impl PhysicalDeviceInfo {
    pub fn processDevice(
        device: vk::PhysicalDevice,
        instance: &ash::Instance,
        surface: &vk::SurfaceKHR,
        surfaceLoader: &Surface,
        requiredExtensions: &[&CStr],
    ) -> Result<Self, Box<dyn Error>> {
        let graphicsQueue = Self::getGraphicsQueue(&device, instance);
        let presentQueue = Self::getPresentQueue(&device, instance, surface, surfaceLoader);

        let properties = unsafe { instance.get_physical_device_properties(device) };
        let discreteGPU = properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU;

        let extensions = unsafe { instance.enumerate_device_extension_properties(device)? };

        let extensionsSupport = requiredExtensions.iter().all(|x| {
            extensions
                .iter()
                .find(|&&y| {
                    let weirdness = unsafe { &*{ x.to_bytes() as *const [u8] as *const [i8] } };

                    let other = &y.extension_name[..weirdness.len()];

                    other.eq(weirdness)
                })
                .is_some()
        });

        let swapchainSupportDetails =
            SwapchainSupportDetails::create(&device, surfaceLoader, surface)?;

        log::info!(
            "\ngraphicsQueue: {}\npresentQueue: {}\ndiscreteGPU: {}\nextensionSupport : {}",
            graphicsQueue.is_some(),
            presentQueue.is_some(),
            discreteGPU,
            extensionsSupport
        );

        let suitable = graphicsQueue.is_some() && presentQueue.is_some() && extensionsSupport;

        Ok(Self {
            device,
            properties,
            graphicsQueue,
            presentQueue,
            suitable,
            swapchainSupportDetails,
        })
    }
    pub fn isSuitable(&self) -> bool {
        self.suitable
    }
    fn getGraphicsQueue(device: &vk::PhysicalDevice, instance: &ash::Instance) -> Option<usize> {
        let props = unsafe { instance.get_physical_device_queue_family_properties(*device) };
        props
            .iter()
            .enumerate()
            .filter_map(|(index, prop)| {
                if prop.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                    Some(index)
                } else {
                    None
                }
            })
            .next()
    }
    fn getPresentQueue(
        device: &vk::PhysicalDevice,
        instance: &ash::Instance,
        surface: &vk::SurfaceKHR,
        surfaceLoader: &Surface,
    ) -> Option<usize> {
        let props = unsafe { instance.get_physical_device_queue_family_properties(*device) };

        props
            .iter()
            .enumerate()
            .filter_map(|(index, prop)| unsafe {
                match surfaceLoader
                    .get_physical_device_surface_support(*device, index as u32, *surface)
                    .unwrap()
                {
                    true => Some(index),
                    false => None,
                }
            })
            .next()
    }
    pub fn toPhysicalDevice(self) -> PhysicalDevice {
        PhysicalDevice {
            graphicsQueueIx: self.graphicsQueue.unwrap(),
            presentQueueIx: self.presentQueue.unwrap(),
            inner: self.device,
            swapchainSupportDetails: self.swapchainSupportDetails,
            properties: self.properties
        }
    }
}
#[derive(Clone)]
pub struct SwapchainSupportDetails {
    capabilities: vk::SurfaceCapabilitiesKHR,
    formats: Vec<vk::SurfaceFormatKHR>,
    presentModes: Vec<vk::PresentModeKHR>,
}
impl SwapchainSupportDetails {
    pub fn create(
        &device: &vk::PhysicalDevice,
        surfaceLoader: &Surface,
        &surface: &vk::SurfaceKHR,
    ) -> Result<Self, Box<dyn Error>> {
        let formats =
            unsafe { surfaceLoader.get_physical_device_surface_formats(device, surface) }?;
        let capabilities =
            unsafe { surfaceLoader.get_physical_device_surface_capabilities(device, surface) }?;
        let presentModes =
            unsafe { surfaceLoader.get_physical_device_surface_present_modes(device, surface) }?;

        Ok(Self {
            capabilities,
            formats,
            presentModes,
        })
    }
    pub fn capabilities(&self) -> &vk::SurfaceCapabilitiesKHR {
        &self.capabilities
    }
    pub fn formats(&self) -> &Vec<vk::SurfaceFormatKHR> {
        &self.formats
    }
    pub fn presentModes(&self) -> &Vec<vk::PresentModeKHR> {
        &self.presentModes
    }
}
pub struct PhysicalDevice {
    inner: vk::PhysicalDevice,
    graphicsQueueIx: usize,
    presentQueueIx: usize,
    swapchainSupportDetails: SwapchainSupportDetails,
    properties: ash::vk::PhysicalDeviceProperties
}
pub struct GPUProperties<'a> {
    name: &'a str,
}
impl<'a> GPUProperties<'a> {
    pub fn name(&self) -> &str {
        self.name
    }
}
impl PhysicalDevice {
    pub fn graphicsQueueIndex(&self) -> usize {
        self.graphicsQueueIx
    }
    pub fn presentQueueIndex(&self) -> usize {
        self.presentQueueIx
    }
    pub fn swapchainSupportDetails(&self) -> &SwapchainSupportDetails {
        &self.swapchainSupportDetails
    }
    pub fn getGPUProperties(&self) -> GPUProperties{

        let name=unsafe { 
            let weirdness = &self.properties.device_name as *const i8;
            CStr::from_ptr(weirdness).to_str().unwrap()
         };
        GPUProperties {
            name,
        }
    }
    pub fn rawDevice(&self) -> vk::PhysicalDevice {
        self.inner
    }
}

impl Drop for PhysicalDevice {
    fn drop(&mut self) {}
}

pub struct Device {
    inner: ash::Device,
}
impl Device {
    pub(super) fn create(
        instance: &ash::Instance,
        physicalDevice: &PhysicalDevice,
    ) -> Result<Self, Box<dyn Error>> {
        const priorities: [f32; 1] = [1.0];

        let mut uniqueQueues = HashSet::new();
        uniqueQueues.insert(physicalDevice.graphicsQueueIndex());
        uniqueQueues.insert(physicalDevice.presentQueueIndex());

        let queueCreateInfos: Vec<_> = uniqueQueues
            .iter()
            .map(|&queue| {
                vk::DeviceQueueCreateInfo::builder()
                    .queue_family_index(queue as u32)
                    .queue_priorities(&priorities)
                    .build()
            })
            .collect();

        let extensions = [Swapchain::name().as_ptr()];

        let deviceCreateInfo = vk::DeviceCreateInfo::builder()
            .enabled_extension_names(&extensions)
            .queue_create_infos(&queueCreateInfos);

        let inner =
            unsafe { instance.create_device(physicalDevice.rawDevice(), &deviceCreateInfo, None)? };

        let graphicsQueue =
            unsafe { inner.get_device_queue(physicalDevice.graphicsQueueIndex() as u32, 0) };
        let presentQueue =
            unsafe { inner.get_device_queue(physicalDevice.presentQueueIndex() as u32, 0) };

        Ok(Self { inner })
    }
    pub fn rawDevice(&self) -> &ash::Device {
        &self.inner
    }
    pub(super) fn getDeviceQueue(&self, index: usize) -> vk::Queue {
        unsafe { self.inner.get_device_queue(index as u32, 0) }
    }
}
impl Drop for Device {
    fn drop(&mut self) {
        unsafe { self.inner.destroy_device(None) }
    }
}
