use ash::vk;

use std::sync::Arc;

use crate::device::DeviceContext;
use crate::utils::find_memorytype_index;

pub struct Texture {
    ctx: Arc<DeviceContext>,
    pub image: vk::Image,
    pub view: vk::ImageView,
    pub memory: vk::DeviceMemory,
    pub format: vk::Format,
    pub extent: vk::Extent2D,
}

impl Texture {
    /// Create a depth/stencil texture sized to `extent`.
    pub fn create_depth(
        ctx: Arc<DeviceContext>,
        extent: vk::Extent2D,
        format: vk::Format, // e.g. vk::Format::D16_UNORM
    ) -> anyhow::Result<Self> {
        let device = &ctx.device;

        let image_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .format(format)
            .extent(extent.into())
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let image = unsafe { device.create_image(&image_info, None)? };
        let mem_req = unsafe { device.get_image_memory_requirements(image) };

        let mem_index = find_memorytype_index(
            &mem_req,
            &ctx.device_memory_properties,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )
        .ok_or_else(|| anyhow::anyhow!("No suitable memory for depth image"))?;

        let memory = unsafe {
            device.allocate_memory(
                &vk::MemoryAllocateInfo::default()
                    .allocation_size(mem_req.size)
                    .memory_type_index(mem_index),
                None,
            )?
        };
        unsafe { device.bind_image_memory(image, memory, 0)? };

        let view = unsafe {
            device.create_image_view(
                &vk::ImageViewCreateInfo::default()
                    .image(image)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(format)
                    .subresource_range(
                        vk::ImageSubresourceRange::default()
                            .aspect_mask(vk::ImageAspectFlags::DEPTH)
                            .level_count(1)
                            .layer_count(1),
                    ),
                None,
            )?
        };

        Ok(Self {
            ctx,
            image,
            view,
            memory,
            format,
            extent,
        })
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            self.ctx.device.destroy_image_view(self.view, None);
            self.ctx.device.free_memory(self.memory, None);
            self.ctx.device.destroy_image(self.image, None);
        }
    }
}
