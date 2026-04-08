use crate::context::find_memorytype_index;

//create a buffer in gpu memory
use super::context::VkContext;
use ash::vk;

#[derive(Default, Debug, Clone)]
pub struct Buffer {
    pub memory: vk::DeviceMemory,
    pub buffer: vk::Buffer,
    pub size: vk::DeviceSize,
    pub requirements: vk::MemoryRequirements,
    ptr: Option<*mut std::ffi::c_void>,
}

impl Buffer {
    pub fn create(
        context: &VkContext,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        flags: vk::MemoryPropertyFlags,
    ) -> Self {
        let buffer_info = vk::BufferCreateInfo::default()
            .size(size as u64)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe { context.device.create_buffer(&buffer_info, None).unwrap() };
        let buffer_memory_requirements =
            unsafe { context.device.get_buffer_memory_requirements(buffer) };

        let buffer_memory_index = find_memorytype_index(
            &buffer_memory_requirements,
            &context.device_memory_properties,
            flags,
        )
        .expect("Unable to find suitable memorytype for the index buffer.");

        let allocate_info = vk::MemoryAllocateInfo {
            allocation_size: buffer_memory_requirements.size,
            memory_type_index: buffer_memory_index,
            ..Default::default()
        };
        let buffer_memory = unsafe {
            context
                .device
                .allocate_memory(&allocate_info, None)
                .unwrap()
        };

        unsafe {
            context
                .device
                .bind_buffer_memory(buffer, buffer_memory, 0)
                .expect("Failed to bind memory");
        }

        Self {
            size,
            requirements: buffer_memory_requirements,
            memory: buffer_memory,
            buffer,
            ptr: None,
        }
    }

    pub fn map_memory<T: Copy>(&mut self, context: &VkContext) {
        self.ptr = Some(
            unsafe {
                context.device.map_memory(
                    self.memory,
                    0,
                    self.requirements.size,
                    vk::MemoryMapFlags::empty(),
                )
            }
            .expect("failed to map memory"),
        );
    }

    pub fn unmap_memory(&mut self, context: &VkContext) {
        unsafe {
            context.device.unmap_memory(self.memory);
        }
        self.ptr = None;
    }

    pub fn copy_to_mapped_memory<T: Copy>(&mut self, context: &VkContext, data: &[T]) {
        if self.ptr.is_none() {
            self.map_memory(context);
        }

        let mapped_ptr = self.ptr.unwrap() as *mut T;
        unsafe {
            mapped_ptr.copy_from_nonoverlapping(data.as_ptr(), data.len());
        }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            context.device.destroy_buffer(self.buffer, None);
            context.device.free_memory(self.memory, None);
        }
    }
}

pub fn create_device_local_buffer<T: Copy>(
    context: &VkContext,
    data: &[T],
    usage: vk::BufferUsageFlags,
) -> Buffer {
    let buffer_size = (data.len() * size_of::<T>()) as vk::DeviceSize;
    Buffer::create(
        context,
        buffer_size,
        usage,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    )
}
