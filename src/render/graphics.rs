use ash::{
    extensions::{
        ext::DebugUtils,
        khr::{Surface, Swapchain},
    },
    version::InstanceV1_0,
    vk::{self, DebugUtilsMessengerEXT},
    Entry, Instance,
};
use ash::{version::EntryV1_0, vk::make_version};
use winit::window::Window;

use std::{borrow::Cow, error::Error, ffi::{CStr, CString}, sync::Arc};

use super::PhysicalDevice;

unsafe extern "system" fn vulkanDebugCallback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;
    let message_id_number: i32 = callback_data.message_id_number as i32;

    let message_id_name = if callback_data.p_message_id_name.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    };

    let message = if callback_data.p_message.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };

    match message_severity {
        // vk::DebugUtilsMessageSeverityFlagsEXT::INFO => {
        //     log::info!("{bold}[Vulkan]{reset} {:?}\n [{}({})]:{}{reset}",
        //     message_type,
        //     message_id_name,
        //     &message_id_number.to_string(),
        //     message,
        //     bold = crossterm::style::Attribute::Bold,
        //     reset = crossterm::style::Attribute::Reset,
        //     );
        // }
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => {
            log::debug!(
                "{bold}[Vulkan]{reset} {:?}\n [{}({})]:{}{reset}",
                message_type,
                message_id_name,
                &message_id_number.to_string(),
                message,
                bold = crossterm::style::Attribute::Bold,
                reset = crossterm::style::Attribute::Reset,
            );
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => {
            log::warn!(
                "{bold}[Vulkan]{reset} {:?}\n [{}({})]:{}{reset}",
                message_type,
                message_id_name,
                &message_id_number.to_string(),
                message,
                bold = crossterm::style::Attribute::Bold,
                reset = crossterm::style::Attribute::Reset,
            );
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => {
            log::error!(
                "{bold}[Vulkan]{reset} {:?}\n [{}({})]:{}{reset}",
                message_type,
                message_id_name,
                &message_id_number.to_string(),
                message,
                bold = crossterm::style::Attribute::Bold,
                reset = crossterm::style::Attribute::Reset,
            );
        }
        _ => {
            // log::info!("{bold}[Vulkan]{reset} {:?}\n [{}({})]:{}{reset}",
            // message_type,
            // message_id_name,
            // &message_id_number.to_string(),
            // message,
            // bold = crossterm::style::Attribute::Bold,
            // reset = crossterm::style::Attribute::Reset,
            // );
        }
    }

    vk::FALSE
}

pub struct Gfx {
    entry: ash::Entry,
    instance: ash::Instance,
    device: Arc<super::Device>,
    pdevice: super::PhysicalDevice,
    swapchain: super::Swapchain,
}
impl Gfx {
    fn getExtensions(window: &Window) -> Vec<*const i8> {
        let mut baseExtensions = ash_window::enumerate_required_extensions(window).unwrap();

        #[cfg(debug_assertions)]
        baseExtensions.push(DebugUtils::name());

        println!("{:?}", baseExtensions);

        baseExtensions.iter().map(|x| x.as_ptr()).collect()
    }
    fn createInstance(entry: &Entry, window: &Window) -> Result<Instance, Box<dyn Error>> {
        unsafe {
            let appName = CString::new("Nuru").unwrap();

            let appInfo = vk::ApplicationInfo::builder()
                .api_version(vk::make_version(1, 0, 0))
                .application_name(&appName)
                .engine_name(&appName)
                .engine_version(make_version(0, 69, 420));

            let layerNames = [CString::new("VK_LAYER_KHRONOS_validation").unwrap()];
            let layerNames: Vec<_> = layerNames.iter().map(|s| s.as_ptr()).collect();

            let extensionNames = Self::getExtensions(window);

            let createInfo = vk::InstanceCreateInfo::builder()
                .application_info(&appInfo)
                .enabled_extension_names(&&extensionNames);

            #[cfg(debug_assertions)]
            let createInfo = createInfo.enabled_layer_names(&layerNames);

            match entry.create_instance(&createInfo, None) {
                Ok(instance) => Ok(instance),
                Err(e) => Err(Box::new(e)),
            }
        }
    }
    fn createDebugMessenger(
        entry: &Entry,
        instance: &Instance,
        callback: vk::PFN_vkDebugUtilsMessengerCallbackEXT,
    ) -> Result<DebugUtilsMessengerEXT, Box<dyn Error>> {
        use vk::DebugUtilsMessageSeverityFlagsEXT as severity;
        let debugInfo = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(
                severity::INFO | severity::ERROR | severity::WARNING | severity::VERBOSE,
            )
            .message_type(vk::DebugUtilsMessageTypeFlagsEXT::all())
            .pfn_user_callback(callback);
        let debugUtilsLoader = DebugUtils::new(entry, instance);

        unsafe { Ok(debugUtilsLoader.create_debug_utils_messenger(&debugInfo, None)?) }
    }
    fn createSurface(
        entry: &Entry,
        instance: &Instance,
        window: &Window,
    ) -> Result<vk::SurfaceKHR, Box<dyn Error>> {
        Ok(unsafe { ash_window::create_surface(entry, instance, window, None)? })
    }
    fn pickPhysicalDevice(
        entry: &Entry,
        instance: &Instance,
        surface: vk::SurfaceKHR,
        surfaceLoader: &Surface,
    ) -> Result<PhysicalDevice, Box<dyn Error>> {
        let devices = unsafe { instance.enumerate_physical_devices()? };
        use super::device::PhysicalDeviceInfo;

        let requiredExtensions = [Swapchain::name()];

        let deviceInfos: Vec<_> = devices
            .iter()
            .map(|device| {
                PhysicalDeviceInfo::processDevice(
                    *device,
                    instance,
                    &surface,
                    &surfaceLoader,
                    &requiredExtensions,
                )
                .unwrap()
            })
            .collect();

        let pickedDeviceInfo = {
            let mut pppppp = None;
            for device in deviceInfos {
                if device.isSuitable() {
                    pppppp.replace(device);
                    break;
                }
            }
            pppppp
        };

        if let Some(deviceInfo) = pickedDeviceInfo {
            Ok(deviceInfo.toPhysicalDevice())
        } else {
            Err("Failed to find a suitable device".into())
        }
    }
    // fn createLogicalDevice(
    //     instance: &Instance,
    //     physicalDevice: vk::PhysicalDevice,
    //     graphicsQueueIx: usize,
    // ) -> Result<ash::Device, Box<dyn Error>> {
    // }
    pub fn new(window: &Window) -> Result<Self, Box<dyn Error>> {
        let entry = unsafe { Entry::new()? };

        entry
            .enumerate_instance_extension_properties()?
            .iter()
            .for_each(|prop| {
                println!("{:?}", unsafe {
                    CStr::from_ptr(prop.extension_name.as_ptr())
                });
            });

        let instance = Self::createInstance(&entry, window)?;
        #[cfg(debug_assertions)]
        let debugCallback =
            Self::createDebugMessenger(&entry, &instance, Some(vulkanDebugCallback))?;

        let surface = Self::createSurface(&entry, &instance, window)?;

        let surfaceLoader = Surface::new(&entry, &instance);

        let pdevice = Self::pickPhysicalDevice(&entry, &instance, surface, &surfaceLoader)?;

        log::debug!("Created physical device");
        log::info!("{}",pdevice.getGPUProperties().name());

        let device = Arc::new(super::Device::create(&instance, &pdevice)?);

        log::debug!("Created logical device");

        let swapchain = super::Swapchain::create(
            &instance,
            &device,
            &pdevice,
            window,
            &surface,
            &surfaceLoader,
        )?;

        log::debug!("Created swapchain");

        Ok(Self {
            entry,
            instance,
            device,
            pdevice,
            swapchain,
        })
    }
}
impl Drop for Gfx {
    fn drop(&mut self) {
        unsafe { self.instance.destroy_instance(None) }
    }
}
