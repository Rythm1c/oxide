use ash::ext::debug_utils;
use ash::{
    Entry, Instance,
    khr::{surface, swapchain},
    vk,
};

use std::{borrow::Cow, default::Default, error::Error, ffi, os::raw::c_char};

use std::ffi::CString;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};

pub struct VkContext {
    pub entry: Entry,
    pub instance: Instance,
    pub debug_utils_loader: debug_utils::Instance,
    pub debug_call_back: vk::DebugUtilsMessengerEXT,
    pub surface_loader: surface::Instance,
    pub swapchain_loader: swapchain::Device,

    pub physical_device: vk::PhysicalDevice,
    pub device_memory_properties: vk::PhysicalDeviceMemoryProperties,
    pub queue_family_index: u32,

    pub device: ash::Device,
    pub present_queue: vk::Queue,

    pub surface: vk::SurfaceKHR,
    pub surface_format: vk::SurfaceFormatKHR,
    pub surface_resolution: vk::Extent2D,

    pub swapchain: vk::SwapchainKHR,
    pub present_images: Vec<vk::Image>,
    pub present_image_views: Vec<vk::ImageView>,

    pub pool: vk::CommandPool,
    pub setup_command_buffer: vk::CommandBuffer,

    pub depth_image: vk::Image,
    pub depth_image_view: vk::ImageView,
    pub depth_image_memory: vk::DeviceMemory,

    pub setup_commands_reuse_fence: vk::Fence,
}

impl VkContext {
    pub fn new(
        app_name: &str,
        window: &winit::window::Window,
        window_width: u32,
        window_height: u32,
    ) -> Result<Self, Box<dyn Error>> {
        let entry = unsafe { Entry::load()? };

        let layer_names = [c"VK_LAYER_KHRONOS_validation"];
        let layers_names_raw: Vec<*const c_char> = layer_names
            .iter()
            .map(|raw_name| raw_name.as_ptr())
            .collect();
        let mut extensions_names =
            ash_window::enumerate_required_extensions(window.display_handle()?.as_raw())
                .unwrap()
                .to_vec();

        extensions_names.push(debug_utils::NAME.as_ptr());

        #[cfg(any(target_os = "macos", target_os = "ios"))]
        {
            extensions_names.push(ash::khr::portability_enumeration::NAME.as_ptr());

            extension_names.push(ash::khr::get_physical_device_properties2::NAME.as_ptr());
        }
        let name = CString::new(app_name).unwrap();
        let appinfo = vk::ApplicationInfo::default()
            .application_name(&name)
            .application_version(0)
            .engine_name(c"egine_core")
            .engine_version(0)
            .api_version(vk::make_api_version(0, 1, 0, 0));

        let create_flags = if cfg!(any(target_os = "macos", target_os = "ios")) {
            vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
        } else {
            vk::InstanceCreateFlags::default()
        };

        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&appinfo)
            .enabled_layer_names(&layers_names_raw)
            .enabled_extension_names(&extensions_names)
            .flags(create_flags);

        let instance: Instance = unsafe {
            entry
                .create_instance(&create_info, None)
                .expect("instance creation error")
        };

        let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            )
            .pfn_user_callback(Some(vulkan_debug_callback));

        let debug_utils_loader = debug_utils::Instance::new(&entry, &instance);
        let debug_call_back = unsafe {
            debug_utils_loader
                .create_debug_utils_messenger(&debug_info, None)
                .unwrap()
        };
        println!("Vulkan instance created successfully");

        let surface = unsafe {
            ash_window::create_surface(
                &entry,
                &instance,
                window.display_handle()?.as_raw(),
                window.window_handle()?.as_raw(),
                None,
            )
            .expect("failed to create surface!")
        };

        let devices = unsafe { instance.enumerate_physical_devices() }
            .expect("Failed to enumerate physical devices");

        let mut physical_device = None;
        let mut queue_family_index = None;
        // Try discrete GPU first
        for device in &devices {
            let props = unsafe { instance.get_physical_device_properties(*device) };
            let family_props =
                unsafe { instance.get_physical_device_queue_family_properties(*device) };
            if props.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
                if let Some((index, _)) = family_props
                    .iter()
                    .enumerate()
                    .find(|&(_, fp)| fp.queue_flags.contains(vk::QueueFlags::GRAPHICS))
                {
                    println!(
                        "Selected discrete GPU: {}",
                        unsafe { std::ffi::CStr::from_ptr(props.device_name.as_ptr()) }
                            .to_str()
                            .unwrap()
                    );
                    physical_device = Some(*device);
                    queue_family_index = Some(index as u32);
                }
            }
        }

        // Fallback: pick any GPU with graphics support
        for device in &devices {
            let props = unsafe { instance.get_physical_device_properties(*device) };
            let family_props =
                unsafe { instance.get_physical_device_queue_family_properties(*device) };
            if let Some((index, _)) = family_props
                .iter()
                .enumerate()
                .find(|&(_, fp)| fp.queue_flags.contains(vk::QueueFlags::GRAPHICS))
            {
                println!(
                    "Selected fallback GPU: {}",
                    unsafe { std::ffi::CStr::from_ptr(props.device_name.as_ptr()) }
                        .to_str()
                        .unwrap()
                );
                physical_device = Some(*device);
                queue_family_index = Some(index as u32);
            }
        }

        if !physical_device.is_some() {
            return Err("No suitable GPU found".into());
        }
        let physical_device = physical_device.unwrap();
        let queue_family_index = queue_family_index.unwrap();

        let queue_priorities = [1.0_f32];
        let queue_info: vk::DeviceQueueCreateInfo<'_> = vk::DeviceQueueCreateInfo::default()
            .queue_family_index(queue_family_index)
            .queue_priorities(&queue_priorities);

        let device_features = vk::PhysicalDeviceFeatures::default();
        let device_extension_names_raw = [
            swapchain::NAME.as_ptr(),
            #[cfg(any(target_os = "macos", target_os = "ios"))]
            ash::khr::portability_subset::NAME.as_ptr(),
        ];
        let create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(std::slice::from_ref(&queue_info))
            .enabled_features(&device_features)
            .enabled_extension_names(&device_extension_names_raw);

        let device = unsafe { instance.create_device(physical_device, &create_info, None) }
            .expect("Failed to create logical device");

        let present_queue = unsafe { device.get_device_queue(queue_family_index, 0) };

        println!(
            "Created logical device with queue family index: {}",
            queue_family_index
        );

        let surface_loader = surface::Instance::new(&entry, &instance);

        let surface_format = unsafe {
            surface_loader
                .get_physical_device_surface_formats(physical_device, surface)
                .unwrap()[0]
        };

        let surface_capabilities = unsafe {
            surface_loader
                .get_physical_device_surface_capabilities(physical_device, surface)
                .unwrap()
        };
        let mut desired_image_count = surface_capabilities.min_image_count + 1;
        if surface_capabilities.max_image_count > 0
            && desired_image_count > surface_capabilities.max_image_count
        {
            desired_image_count = surface_capabilities.max_image_count;
        }
        let surface_resolution = match surface_capabilities.current_extent.width {
            u32::MAX => vk::Extent2D {
                width: window_width,
                height: window_height,
            },
            _ => surface_capabilities.current_extent,
        };
        let pre_transform = if surface_capabilities
            .supported_transforms
            .contains(vk::SurfaceTransformFlagsKHR::IDENTITY)
        {
            vk::SurfaceTransformFlagsKHR::IDENTITY
        } else {
            surface_capabilities.current_transform
        };
        let present_modes = unsafe {
            surface_loader
                .get_physical_device_surface_present_modes(physical_device, surface)
                .unwrap()
        };
        let present_mode = present_modes
            .iter()
            .cloned()
            .find(|&mode| mode == vk::PresentModeKHR::MAILBOX)
            .unwrap_or(vk::PresentModeKHR::FIFO);
        let swapchain_loader = swapchain::Device::new(&instance, &device);

        let swapchain_create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(surface)
            .min_image_count(desired_image_count)
            .image_color_space(surface_format.color_space)
            .image_format(surface_format.format)
            .image_extent(surface_resolution)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(pre_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .image_array_layers(1);

        let swapchain = unsafe {
            swapchain_loader
                .create_swapchain(&swapchain_create_info, None)
                .unwrap()
        };

        let pool_create_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(queue_family_index);

        let pool = unsafe { device.create_command_pool(&pool_create_info, None).unwrap() };

        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::default()
            .command_buffer_count(1)
            .command_pool(pool)
            .level(vk::CommandBufferLevel::PRIMARY);

        let command_buffers = unsafe {
            device
                .allocate_command_buffers(&command_buffer_allocate_info)
                .unwrap()
        };
        let setup_command_buffer = command_buffers[0];

        let present_images = unsafe { swapchain_loader.get_swapchain_images(swapchain).unwrap() };
        let present_image_views: Vec<vk::ImageView> = present_images
            .iter()
            .map(|&image| {
                let create_view_info = vk::ImageViewCreateInfo::default()
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(surface_format.format)
                    .components(vk::ComponentMapping {
                        r: vk::ComponentSwizzle::R,
                        g: vk::ComponentSwizzle::G,
                        b: vk::ComponentSwizzle::B,
                        a: vk::ComponentSwizzle::A,
                    })
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    })
                    .image(image);
                unsafe { device.create_image_view(&create_view_info, None).unwrap() }
            })
            .collect();
        let device_memory_properties =
            unsafe { instance.get_physical_device_memory_properties(physical_device) };
        let depth_image_create_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .format(vk::Format::D16_UNORM)
            .extent(surface_resolution.into())
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let depth_image = unsafe { device.create_image(&depth_image_create_info, None).unwrap() };
        let depth_image_memory_req = unsafe { device.get_image_memory_requirements(depth_image) };
        let depth_image_memory_index = find_memorytype_index(
            &depth_image_memory_req,
            &device_memory_properties,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )
        .expect("Unable to find suitable memory index for depth image.");

        let depth_image_allocate_info = vk::MemoryAllocateInfo::default()
            .allocation_size(depth_image_memory_req.size)
            .memory_type_index(depth_image_memory_index);

        let depth_image_memory = unsafe {
            device
                .allocate_memory(&depth_image_allocate_info, None)
                .unwrap()
        };

        unsafe {
            device
                .bind_image_memory(depth_image, depth_image_memory, 0)
                .expect("Unable to bind depth image memory")
        };

        let fence_create_info =
            vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);


        let setup_commands_reuse_fence = unsafe {
            device
                .create_fence(&fence_create_info, None)
                .expect("Create fence failed.")
        };

        record_submit_commandbuffer(
            &device,
            setup_command_buffer,
            setup_commands_reuse_fence,
            present_queue,
            &[],
            &[],
            &[],
            |device, setup_command_buffer| {
                let layout_transition_barriers = vk::ImageMemoryBarrier::default()
                    .image(depth_image)
                    .dst_access_mask(
                        vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ
                            | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                    )
                    .new_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                    .old_layout(vk::ImageLayout::UNDEFINED)
                    .subresource_range(
                        vk::ImageSubresourceRange::default()
                            .aspect_mask(vk::ImageAspectFlags::DEPTH)
                            .layer_count(1)
                            .level_count(1),
                    );

                unsafe {
                    device.cmd_pipeline_barrier(
                        setup_command_buffer,
                        vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                        vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
                        vk::DependencyFlags::empty(),
                        &[],
                        &[],
                        &[layout_transition_barriers],
                    )
                };
            },
        );

        let depth_image_view_info = vk::ImageViewCreateInfo::default()
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(vk::ImageAspectFlags::DEPTH)
                    .level_count(1)
                    .layer_count(1),
            )
            .image(depth_image)
            .format(depth_image_create_info.format)
            .view_type(vk::ImageViewType::TYPE_2D);

        let depth_image_view = unsafe {
            device
                .create_image_view(&depth_image_view_info, None)
                .unwrap()
        };


        Ok(Self {
            entry,
            instance,
            debug_utils_loader,
            debug_call_back,
            surface_loader,
            swapchain_loader,

            physical_device,
            device_memory_properties,
            queue_family_index,

            device,
            present_queue,

            surface,
            surface_format,
            surface_resolution,
            swapchain,
            present_images,
            present_image_views,

            depth_image,
            depth_image_view,
            depth_image_memory,

            pool,
            setup_command_buffer,
            setup_commands_reuse_fence,
        })
    }

    pub fn destroy(&mut self) {
        unsafe {
            self.device.device_wait_idle().unwrap();
            self.device
                .destroy_fence(self.setup_commands_reuse_fence, None);
            self.device.free_memory(self.depth_image_memory, None);
            self.device.destroy_image_view(self.depth_image_view, None);
            self.device.destroy_image(self.depth_image, None);
            for &image_view in self.present_image_views.iter() {
                self.device.destroy_image_view(image_view, None);
            }
            self.device.destroy_command_pool(self.pool, None);
            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None);
            self.device.destroy_device(None);
            self.surface_loader.destroy_surface(self.surface, None);
            self.debug_utils_loader
                .destroy_debug_utils_messenger(self.debug_call_back, None);
            self.instance.destroy_instance(None);
        }
    }
}

unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT<'_>,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let callback_data = unsafe { &*p_callback_data };

    let message_id_number = callback_data.message_id_number;

    let message_id_name = if callback_data.p_message_id_name.is_null() {
        Cow::from("")
    } else {
        unsafe { ffi::CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy() }
    };

    let message = if callback_data.p_message.is_null() {
        Cow::from("")
    } else {
        unsafe { ffi::CStr::from_ptr(callback_data.p_message).to_string_lossy() }
    };

    println!(
        "{message_severity:?}:\n{message_type:?} [{message_id_name} ({message_id_number})] : {message}\n",
    );

    vk::FALSE
}

pub fn find_memorytype_index(
    memory_req: &vk::MemoryRequirements,
    memory_prop: &vk::PhysicalDeviceMemoryProperties,
    flags: vk::MemoryPropertyFlags,
) -> Option<u32> {
    memory_prop.memory_types[..memory_prop.memory_type_count as _]
        .iter()
        .enumerate()
        .find(|(index, memory_type)| {
            (1 << index) & memory_req.memory_type_bits != 0
                && memory_type.property_flags & flags == flags
        })
        .map(|(index, _memory_type)| index as _)
}

#[allow(clippy::too_many_arguments)]
pub fn record_submit_commandbuffer<F: FnOnce(&ash::Device, vk::CommandBuffer)>(
    device: &ash::Device,
    command_buffer: vk::CommandBuffer,
    command_buffer_reuse_fence: vk::Fence,
    submit_queue: vk::Queue,
    wait_mask: &[vk::PipelineStageFlags],
    wait_semaphores: &[vk::Semaphore],
    signal_semaphores: &[vk::Semaphore],
    f: F,
) {
    unsafe {
        device
            .wait_for_fences(&[command_buffer_reuse_fence], true, u64::MAX)
            .expect("Wait for fence failed.");

        device
            .reset_fences(&[command_buffer_reuse_fence])
            .expect("Reset fences failed.");

        device
            .reset_command_buffer(
                command_buffer,
                vk::CommandBufferResetFlags::RELEASE_RESOURCES,
            )
            .expect("Reset command buffer failed.");

        let command_buffer_begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        device
            .begin_command_buffer(command_buffer, &command_buffer_begin_info)
            .expect("Begin commandbuffer");
        f(device, command_buffer);
        device
            .end_command_buffer(command_buffer)
            .expect("End commandbuffer");

        let command_buffers = vec![command_buffer];

        let submit_info = vk::SubmitInfo::default()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_mask)
            .command_buffers(&command_buffers)
            .signal_semaphores(signal_semaphores);

        device
            .queue_submit(submit_queue, &[submit_info], command_buffer_reuse_fence)
            .expect("queue submit failed.");
    }
}
