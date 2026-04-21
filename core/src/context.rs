use ash::ext::debug_utils;
use ash::{
    Entry, Instance,
    khr::{surface, swapchain},
    vk,
};

use std::ffi::CString;
use std::sync::Arc;
use std::{borrow::Cow, ffi, os::raw::c_char};

use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use crate::device::DeviceContext;
use crate::swapchain::SwapchainContext;
use crate::texture::Texture;

pub struct VkContext {
    // Keep entry + debug alive for the full lifetime
    pub entry: Entry,
    pub instance: Instance,
    pub debug_utils_loader: debug_utils::Instance,
    pub debug_callback: vk::DebugUtilsMessengerEXT,

    pub surface_loader: surface::Instance,
    pub surface: vk::SurfaceKHR,

    // Core device — shared with buffers / textures
    pub device_ctx: Arc<DeviceContext>,

    // Swapchain — recreated on resize
    pub swapchain_ctx: SwapchainContext,

    // Depth buffer (Texture owns image + view + memory)
    pub depth_texture: Texture,

    // Setup command buffer / fence used for one-shot transfers during init
    pub setup_command_buffer: vk::CommandBuffer,
    pub setup_commands_reuse_fence: vk::Fence,
}

impl VkContext {
    pub fn new(
        app_name: &str,
        window: &winit::window::Window,
        window_width: u32,
        window_height: u32,
    ) -> anyhow::Result<Self> {
        // --- Entry & Instance -------------------------------------------
        let entry = unsafe { Entry::load()? };

        let layer_names = [c"VK_LAYER_KHRONOS_validation"];
        let layers_raw: Vec<*const c_char> = layer_names.iter().map(|n| n.as_ptr()).collect();

        let mut extensions =
            ash_window::enumerate_required_extensions(window.display_handle()?.as_raw())
                .unwrap()
                .to_vec();
        extensions.push(debug_utils::NAME.as_ptr());

        #[cfg(any(target_os = "macos", target_os = "ios"))]
        {
            extensions.push(ash::khr::portability_enumeration::NAME.as_ptr());
            extensions.push(ash::khr::get_physical_device_properties2::NAME.as_ptr());
        }

        let app_name_cstr = CString::new(app_name)?;
        let appinfo = vk::ApplicationInfo::default()
            .application_name(&app_name_cstr)
            .application_version(0)
            .engine_name(c"engine_core")
            .engine_version(0)
            .api_version(vk::make_api_version(0, 1, 0, 0));

        let create_flags = if cfg!(any(target_os = "macos", target_os = "ios")) {
            vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
        } else {
            vk::InstanceCreateFlags::default()
        };

        let instance: Instance = unsafe {
            entry.create_instance(
                &vk::InstanceCreateInfo::default()
                    .application_info(&appinfo)
                    .enabled_layer_names(&layers_raw)
                    .enabled_extension_names(&extensions)
                    .flags(create_flags),
                None,
            )?
        };

        // --- Debug utils ------------------------------------------------
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
        let debug_callback =
            unsafe { debug_utils_loader.create_debug_utils_messenger(&debug_info, None)? };

        // --- Surface ----------------------------------------------------
        let surface_loader = surface::Instance::new(&entry, &instance);
        let surface = unsafe {
            ash_window::create_surface(
                &entry,
                &instance,
                window.display_handle()?.as_raw(),
                window.window_handle()?.as_raw(),
                None,
            )?
        };

        // --- Physical device & queue ------------------------------------
        let (physical_device, graphics_queue_index) =
            pick_physical_device(&instance, &surface_loader, surface)?;

        let queue_priorities = [1.0_f32];
        let queue_info = vk::DeviceQueueCreateInfo::default()
            .queue_family_index(graphics_queue_index)
            .queue_priorities(&queue_priorities);

        let device_extension_names = [
            swapchain::NAME.as_ptr(),
            #[cfg(any(target_os = "macos", target_os = "ios"))]
            ash::khr::portability_subset::NAME.as_ptr(),
        ];

        let device = unsafe {
            instance.create_device(
                physical_device,
                &vk::DeviceCreateInfo::default()
                    .queue_create_infos(std::slice::from_ref(&queue_info))
                    .enabled_features(&vk::PhysicalDeviceFeatures::default())
                    .enabled_extension_names(&device_extension_names),
                None,
            )?
        };

        let graphics_queue = unsafe { device.get_device_queue(graphics_queue_index, 0) };
        println!("Graphics queue family index: {}", graphics_queue_index);

        // --- Command pool -----------------------------------------------
        let pool = unsafe {
            device.create_command_pool(
                &vk::CommandPoolCreateInfo::default()
                    .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                    .queue_family_index(graphics_queue_index),
                None,
            )?
        };

        // --- DeviceContext (Arc — shared with Buffer / Texture) ---------
        let device_ctx = Arc::new(DeviceContext::new(
            device.clone(),
            physical_device,
            instance.clone(),
            graphics_queue,
            graphics_queue_index,
            pool,
        )?);

        // --- Setup command buffer ---------------------------------------
        let setup_command_buffer = unsafe {
            device.allocate_command_buffers(
                &vk::CommandBufferAllocateInfo::default()
                    .command_buffer_count(1)
                    .command_pool(pool)
                    .level(vk::CommandBufferLevel::PRIMARY),
            )?[0]
        };

        let setup_commands_reuse_fence = unsafe {
            device.create_fence(
                &vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED),
                None,
            )?
        };

        // --- Swapchain --------------------------------------------------
        let swapchain_ctx = SwapchainContext::new(
            &instance,
            &device,
            physical_device,
            surface,
            &surface_loader,
            window_width,
            window_height,
        )?;

        // --- Depth texture ----------------------------------------------
        let depth_texture = Texture::create_depth(
            Arc::clone(&device_ctx),
            swapchain_ctx.surface_resolution,
            vk::Format::D16_UNORM,
        )?;

        // Transition depth image layout
        record_submit_commandbuffer(
            &device,
            setup_command_buffer,
            setup_commands_reuse_fence,
            graphics_queue,
            &[],
            &[],
            &[],
            |device, cmd| {
                let barrier = vk::ImageMemoryBarrier::default()
                    .image(depth_texture.image)
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
                        cmd,
                        vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                        vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
                        vk::DependencyFlags::empty(),
                        &[],
                        &[],
                        &[barrier],
                    );
                }
            },
        );

        println!("VkContext initialised successfully.");

        Ok(Self {
            entry,
            instance,
            debug_utils_loader,
            debug_callback,
            surface_loader,
            surface,
            device_ctx,
            swapchain_ctx,
            depth_texture,
            setup_command_buffer,
            setup_commands_reuse_fence,
        })
    }

    /// Shorthand helpers so callers don't always have to reach into device_ctx.
    #[inline]
    pub fn device(&self) -> &ash::Device {
        &self.device_ctx.device
    }
    #[inline]
    pub fn surface_format(&self) -> vk::SurfaceFormatKHR {
        self.swapchain_ctx.surface_format
    }
    #[inline]
    pub fn surface_resolution(&self) -> vk::Extent2D {
        self.swapchain_ctx.surface_resolution
    }
    #[inline]
    pub fn swapchain(&self) -> vk::SwapchainKHR {
        self.swapchain_ctx.swapchain
    }
    #[inline]
    pub fn present_image_views(&self) -> &[vk::ImageView] {
        &self.swapchain_ctx.present_image_views
    }
    #[inline]
    pub fn depth_image_view(&self) -> vk::ImageView {
        self.depth_texture.view
    }
    #[inline]
    pub fn present_queue(&self) -> vk::Queue {
        self.device_ctx.graphics_queue
    }
    #[inline]
    pub fn swapchain_loader(&self) -> &swapchain::Device {
        &self.swapchain_ctx.swapchain_loader
    }

    pub fn destroy(&mut self) {
        unsafe {
            self.device().device_wait_idle().unwrap();
            self.device()
                .destroy_fence(self.setup_commands_reuse_fence, None);

            // depth_texture Drop handles image/view/memory cleanup
            // (but we need to drop it before the device is destroyed)
            // We rely on field drop order: depth_texture before device_ctx.
            // In Rust, fields drop in declaration order (top to bottom).

            self.swapchain_ctx.destroy(self.device());
            self.device()
                .destroy_command_pool(self.device_ctx.pool, None);
            self.device().destroy_device(None);
            self.surface_loader.destroy_surface(self.surface, None);
            self.debug_utils_loader
                .destroy_debug_utils_messenger(self.debug_callback, None);
            self.instance.destroy_instance(None);
        }
    }
}

impl Drop for VkContext {
    fn drop(&mut self) {
        self.destroy();
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn pick_physical_device(
    instance: &Instance,
    surface_loader: &surface::Instance,
    surface: vk::SurfaceKHR,
) -> anyhow::Result<(vk::PhysicalDevice, u32)> {
    let devices = unsafe { instance.enumerate_physical_devices()? };

    // Prefer discrete GPU, fall back to anything with graphics
    let mut fallback: Option<(vk::PhysicalDevice, u32)> = None;

    for pdev in devices {
        let props = unsafe { instance.get_physical_device_properties(pdev) };
        let families = unsafe { instance.get_physical_device_queue_family_properties(pdev) };

        let graphics_index = families.iter().enumerate().find_map(|(i, fp)| {
            if fp.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                Some(i as u32)
            } else {
                None
            }
        });

        if let Some(idx) = graphics_index {
            let name = unsafe { ffi::CStr::from_ptr(props.device_name.as_ptr()) }
                .to_str()
                .unwrap_or("unknown");

            if props.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
                println!("Selected discrete GPU: {}", name);
                return Ok((pdev, idx));
            }
            if fallback.is_none() {
                println!("Found fallback GPU: {}", name);
                fallback = Some((pdev, idx));
            }
        }
    }

    fallback.ok_or_else(|| anyhow::anyhow!("No suitable GPU found"))
}

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
            .expect("Wait for fence failed");
        device
            .reset_fences(&[command_buffer_reuse_fence])
            .expect("Reset fences failed");
        device
            .reset_command_buffer(
                command_buffer,
                vk::CommandBufferResetFlags::RELEASE_RESOURCES,
            )
            .expect("Reset command buffer failed");

        device
            .begin_command_buffer(
                command_buffer,
                &vk::CommandBufferBeginInfo::default()
                    .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
            )
            .expect("Begin command buffer failed");

        f(device, command_buffer);

        device
            .end_command_buffer(command_buffer)
            .expect("End command buffer failed");

        device
            .queue_submit(
                submit_queue,
                &[vk::SubmitInfo::default()
                    .wait_semaphores(wait_semaphores)
                    .wait_dst_stage_mask(wait_mask)
                    .command_buffers(&[command_buffer])
                    .signal_semaphores(signal_semaphores)],
                command_buffer_reuse_fence,
            )
            .expect("Queue submit failed");
    }
}

unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT<'_>,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let data = unsafe { &*p_callback_data };

    let id_name = if data.p_message_id_name.is_null() {
        Cow::from("")
    } else {
        unsafe { ffi::CStr::from_ptr(data.p_message_id_name).to_string_lossy() }
    };

    let message = if data.p_message.is_null() {
        Cow::from("")
    } else {
        unsafe { ffi::CStr::from_ptr(data.p_message).to_string_lossy() }
    };

    println!(
        "{message_severity:?}:\n{message_type:?} [{id_name} ({})] : {message}\n",
        data.message_id_number
    );

    vk::FALSE
}
