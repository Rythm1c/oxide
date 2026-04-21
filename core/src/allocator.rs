// src/allocator.rs
//
// Thin wrapper around `gpu-allocator` (a popular, well-maintained Vulkan
// memory allocator for Rust). If you prefer vk-mem or a custom allocator,
// swap out the internals here — Buffer never sees the difference.

use ash::{Device, Instance, vk};
use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::{
    Allocation as GpuAllocation, AllocationCreateDesc, AllocationScheme, Allocator as GpuAllocator,
    AllocatorCreateDesc,
};

pub use gpu_allocator::MemoryLocation as GpuMemoryLocation;

// Re-export so callers don't need gpu-allocator in scope
pub type Allocation = GpuAllocation;

pub struct AllocatorCreateInfo {
    pub instance: Instance,
    pub device: Device,
    pub physical_device: vk::PhysicalDevice,
}

pub struct Allocator {
    inner: GpuAllocator,
}

impl Allocator {
    pub fn new(info: AllocatorCreateInfo) -> anyhow::Result<Self> {
        let inner = GpuAllocator::new(&AllocatorCreateDesc {
            instance: info.instance,
            device: info.device,
            physical_device: info.physical_device,
            debug_settings: Default::default(),
            buffer_device_address: false,
            allocation_sizes: Default::default(),
        })?;
        Ok(Self { inner })
    }

    pub fn allocate(
        &mut self,
        name: &str,
        requirements: vk::MemoryRequirements,
        location: MemoryLocation,
    ) -> anyhow::Result<Allocation> {
        let allocation = self.inner.allocate(&AllocationCreateDesc {
            name,
            requirements,
            location,
            linear: true,
            allocation_scheme: AllocationScheme::GpuAllocatorManaged,
        })?;
        Ok(allocation)
    }

    pub fn free(&mut self, allocation: Allocation) -> anyhow::Result<()> {
        self.inner.free(allocation)?;
        Ok(())
    }
}
