use ash::vk;

use std::sync::Arc;

use crate::device::DeviceContext;
use crate::utils::find_memorytype_index;

/// Defines the type and usage of a texture
#[derive(Debug, Clone, Copy)]
pub enum TextureType {
    /// Depth/stencil attachment
    Depth,
    /// Color/color attachment texture
    Color,
    /// Storage/compute texture
    Storage,
    /// Sampled texture (read-only)
    Sampled,
}

impl TextureType {
    /// Get the Vulkan image usage flags for this texture type
    fn usage_flags(&self) -> vk::ImageUsageFlags {
        match self {
            TextureType::Depth => vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            TextureType::Color => vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST,
            TextureType::Storage => vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::TRANSFER_DST,
            TextureType::Sampled => vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST,
        }
    }

    /// Get the image aspect flags for this texture type
    fn aspect_flags(&self) -> vk::ImageAspectFlags {
        match self {
            TextureType::Depth => vk::ImageAspectFlags::DEPTH,
            TextureType::Color => vk::ImageAspectFlags::COLOR,
            TextureType::Storage => vk::ImageAspectFlags::COLOR,
            TextureType::Sampled => vk::ImageAspectFlags::COLOR,
        }
    }
}

pub struct Texture {
    ctx: Arc<DeviceContext>,
    pub image: vk::Image,
    pub view: vk::ImageView,
    pub memory: vk::DeviceMemory,
    pub format: vk::Format,
    pub extent: vk::Extent2D,
    pub texture_type: TextureType,
}

impl Texture {
    /// Create a texture of the specified type
    pub fn new(
        ctx: Arc<DeviceContext>,
        texture_type: TextureType,
        extent: vk::Extent2D,
        format: vk::Format,
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
            .usage(texture_type.usage_flags())
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let image = unsafe { device.create_image(&image_info, None)? };
        let mem_req = unsafe { device.get_image_memory_requirements(image) };

        let mem_index = find_memorytype_index(
            &mem_req,
            &ctx.device_memory_properties,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )
        .ok_or_else(|| {
            anyhow::anyhow!("No suitable memory for {:?} texture", texture_type)
        })?;

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
                            .aspect_mask(texture_type.aspect_flags())
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
            texture_type,
        })
    }

    /// Create a depth/stencil texture (convenience method)
    pub fn create_depth(
        ctx: Arc<DeviceContext>,
        extent: vk::Extent2D,
        format: vk::Format,
    ) -> anyhow::Result<Self> {
        Self::new(ctx, TextureType::Depth, extent, format)
    }

    /// Create a color texture (convenience method)
    pub fn create_color(
        ctx: Arc<DeviceContext>,
        extent: vk::Extent2D,
        format: vk::Format,
    ) -> anyhow::Result<Self> {
        Self::new(ctx, TextureType::Color, extent, format)
    }

    /// Create a storage texture (convenience method)
    pub fn create_storage(
        ctx: Arc<DeviceContext>,
        extent: vk::Extent2D,
        format: vk::Format,
    ) -> anyhow::Result<Self> {
        Self::new(ctx, TextureType::Storage, extent, format)
    }

    /// Create a sampled texture (convenience method)
    pub fn create_sampled(
        ctx: Arc<DeviceContext>,
        extent: vk::Extent2D,
        format: vk::Format,
    ) -> anyhow::Result<Self> {
        Self::new(ctx, TextureType::Sampled, extent, format)
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
