use ash::{
    Instance,
    khr::{surface, swapchain},
    vk,
};

pub struct SwapchainContext {
    pub swapchain_loader: swapchain::Device,
    pub swapchain: vk::SwapchainKHR,

    pub surface_format: vk::SurfaceFormatKHR,
    pub surface_resolution: vk::Extent2D,

    pub present_images: Vec<vk::Image>,
    pub present_image_views: Vec<vk::ImageView>,
}

impl SwapchainContext {
    fn select_best_surface_format(formats: &[vk::SurfaceFormatKHR]) -> vk::SurfaceFormatKHR {
        // Preferred format priority list
        let preferred_formats = vec![
            (vk::Format::B8G8R8A8_SRGB, vk::ColorSpaceKHR::SRGB_NONLINEAR),
            (vk::Format::R8G8B8A8_SRGB, vk::ColorSpaceKHR::SRGB_NONLINEAR),
            (vk::Format::B8G8R8_SRGB, vk::ColorSpaceKHR::SRGB_NONLINEAR),
            (vk::Format::R8G8B8_SRGB, vk::ColorSpaceKHR::SRGB_NONLINEAR),
        ];

        // Try to find a preferred format
        for (preferred_format, preferred_color_space) in preferred_formats {
            if let Some(&format) = formats
                .iter()
                .find(|f| f.format == preferred_format && f.color_space == preferred_color_space)
            {
                return format;
            }
        }

        // Fallback: look for any SRGB format
        if let Some(&format) = formats
            .iter()
            .find(|f| f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR)
        {
            return format;
        }

        // Last resort: use the first available format
        formats[0]
    }

    pub fn new(
        instance: &Instance,
        device: &ash::Device,
        physical_device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
        surface_loader: &surface::Instance,
        window_width: u32,
        window_height: u32,
    ) -> anyhow::Result<Self> {
        let surface_formats = unsafe {
            surface_loader.get_physical_device_surface_formats(physical_device, surface)?
        };
        let surface_format = Self::select_best_surface_format(&surface_formats);

        let surface_capabilities = unsafe {
            surface_loader.get_physical_device_surface_capabilities(physical_device, surface)?
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
            surface_loader.get_physical_device_surface_present_modes(physical_device, surface)?
        };
        let present_mode = present_modes
            .iter()
            .copied()
            .find(|&m| m == vk::PresentModeKHR::MAILBOX)
            .unwrap_or(vk::PresentModeKHR::FIFO);

        let swapchain_loader = swapchain::Device::new(instance, device);

        let swapchain = unsafe {
            swapchain_loader.create_swapchain(
                &vk::SwapchainCreateInfoKHR::default()
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
                    .image_array_layers(1),
                None,
            )?
        };

        let present_images = unsafe { swapchain_loader.get_swapchain_images(swapchain)? };
        let present_image_views = present_images
            .iter()
            .map(|&image| {
                let info = vk::ImageViewCreateInfo::default()
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
                unsafe { device.create_image_view(&info, None) }
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            swapchain_loader,
            swapchain,
            surface_format,
            surface_resolution,
            present_images,
            present_image_views,
        })
    }

    pub fn destroy(&self, device: &ash::Device) {
        unsafe {
            for &view in &self.present_image_views {
                device.destroy_image_view(view, None);
            }
            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None);
        }
    }
}
