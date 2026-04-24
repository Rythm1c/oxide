use ash::{Instance, vk};

use std::sync::Mutex;

use crate::allocator::{Allocator, AllocatorCreateInfo};
pub struct DeviceContext {
    pub device: ash::Device,
    pub physical_device: vk::PhysicalDevice,
    pub instance: Instance, // cloned ref for allocator creation
    pub device_memory_properties: vk::PhysicalDeviceMemoryProperties,

    pub graphics_queue: vk::Queue,
    pub graphics_queue_index: u32,

/*     pub allocator: Mutex<Allocator>, */
    pub pool: vk::CommandPool,
}

impl DeviceContext {
    /// Internal — called by VkContext::new. Not pub(crate) so the module
    /// boundary is clean; callers always go through VkContext.
    pub fn new(
        device: ash::Device,
        physical_device: vk::PhysicalDevice,
        instance: Instance,
        graphics_queue: vk::Queue,
        graphics_queue_index: u32,
        pool: vk::CommandPool,
    ) -> anyhow::Result<Self> {
        let device_memory_properties =
            unsafe { instance.get_physical_device_memory_properties(physical_device) };


        Ok(Self {
            device,
            physical_device,
            instance,
            device_memory_properties,
            graphics_queue,
            graphics_queue_index,
            pool,
        })
    }
}
