use super::device::DeviceContext;
use crate::allocator::Allocation;

use ash::vk;
use gpu_allocator::MemoryLocation;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// BufferUsage flags
// ---------------------------------------------------------------------------

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct BufferUsage: u32 {
        const VERTEX       = 0b0000_0001;
        const INDEX        = 0b0000_0010;
        const UNIFORM      = 0b0000_0100;
        const STORAGE      = 0b0000_1000;
        const TRANSFER_SRC = 0b0001_0000;
        const TRANSFER_DST = 0b0010_0000;
    }
}

impl BufferUsage {
    pub fn to_vk_usage(self) -> vk::BufferUsageFlags {
        let mut flags = vk::BufferUsageFlags::empty();
        if self.contains(Self::VERTEX) {
            flags |= vk::BufferUsageFlags::VERTEX_BUFFER;
        }
        if self.contains(Self::INDEX) {
            flags |= vk::BufferUsageFlags::INDEX_BUFFER;
        }
        if self.contains(Self::UNIFORM) {
            flags |= vk::BufferUsageFlags::UNIFORM_BUFFER;
        }
        if self.contains(Self::STORAGE) {
            flags |= vk::BufferUsageFlags::STORAGE_BUFFER;
        }
        if self.contains(Self::TRANSFER_SRC) {
            flags |= vk::BufferUsageFlags::TRANSFER_SRC;
        }
        if self.contains(Self::TRANSFER_DST) {
            flags |= vk::BufferUsageFlags::TRANSFER_DST;
        }
        flags
    }

    /// CPU-visible buffers: uniforms and storage need frequent CPU writes.
    /// Everything else (vertex, index) prefers GPU-only memory.
    fn preferred_memory_location(self) -> MemoryLocation {
        if self.intersects(Self::UNIFORM | Self::STORAGE) {
            MemoryLocation::CpuToGpu
        } else {
            MemoryLocation::GpuOnly
        }
    }

    fn is_cpu_visible(self) -> bool {
        self.preferred_memory_location() == MemoryLocation::CpuToGpu
    }
}

// ---------------------------------------------------------------------------
// Buffer
// ---------------------------------------------------------------------------

pub struct Buffer {
    ctx: Arc<DeviceContext>,
    pub(crate) raw: vk::Buffer,
    allocation: Option<Allocation>, // Option so we can move out in Drop
    pub size: vk::DeviceSize,
    pub usage: BufferUsage,
}

impl Buffer {
    /// Allocate an uninitialised buffer of `size` bytes.
    pub fn new(
        ctx: Arc<DeviceContext>,
        size: vk::DeviceSize,
        usage: BufferUsage,
    ) -> anyhow::Result<Self> {
        Self::new_internal(ctx, size, usage)
    }

    /// Allocate a buffer and immediately upload `data`.
    ///
    /// - CPU-visible buffers (UNIFORM / STORAGE): mapped and written directly.
    /// - GPU-only buffers (VERTEX / INDEX): uploaded via a staging buffer +
    ///   one-shot transfer command.
    pub fn new_with_data<T: Copy>(
        ctx: Arc<DeviceContext>,
        data: &[T],
        usage: BufferUsage,
    ) -> anyhow::Result<Self> {
        let size = std::mem::size_of_val(data) as vk::DeviceSize;
        assert!(size > 0, "Buffer::new_with_data called with empty slice");

        if usage.is_cpu_visible() {
            // Direct map — no staging needed
            let mut buffer = Self::new_internal(Arc::clone(&ctx), size, usage)?;
            buffer.write(data)?;
            Ok(buffer)
        } else {
            // Actually the cleanest fix: expose an explicit memory location
            // parameter in new_internal (done below via new_staging).
            let mut staging = Self::new_staging(Arc::clone(&ctx), size)?;
            staging.write(data)?;

            let dst =
                Self::new_internal(Arc::clone(&ctx), size, usage | BufferUsage::TRANSFER_DST)?;

            copy_buffer(Arc::clone(&ctx), staging.raw, dst.raw, size)?;
            Ok(dst)
        }
    }

    /// Map and write `data` into a CPU-visible buffer.
    pub fn write<T: Copy>(&mut self, data: &[T]) -> anyhow::Result<()> {
        let alloc = self
            .allocation
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Buffer has no allocation"))?;

        let dst = alloc.mapped_slice_mut().ok_or_else(|| {
            anyhow::anyhow!(
                "Buffer is not CPU-visible; cannot write directly. \
                Use a staging buffer for GPU-only buffers."
            )
        })?;

        let src = unsafe {
            std::slice::from_raw_parts(data.as_ptr() as *const u8, std::mem::size_of_val(data))
        };

        dst[..src.len()].copy_from_slice(src);
        Ok(())
    }

    // ---- private helpers -----------------------------------------------

    fn new_internal(
        ctx: Arc<DeviceContext>,
        size: vk::DeviceSize,
        usage: BufferUsage,
    ) -> anyhow::Result<Self> {
        Self::new_with_location(ctx, size, usage, usage.preferred_memory_location())
    }

    /// A CpuToGpu staging buffer (TRANSFER_SRC only, always CPU-visible).
    fn new_staging(ctx: Arc<DeviceContext>, size: vk::DeviceSize) -> anyhow::Result<Self> {
        Self::new_with_location(
            ctx,
            size,
            BufferUsage::TRANSFER_SRC,
            MemoryLocation::CpuToGpu,
        )
    }

    fn new_with_location(
        ctx: Arc<DeviceContext>,
        size: vk::DeviceSize,
        usage: BufferUsage,
        location: MemoryLocation,
    ) -> anyhow::Result<Self> {
        let buffer_info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(usage.to_vk_usage())
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let raw = unsafe { ctx.device.create_buffer(&buffer_info, None)? };
        let requirements = unsafe { ctx.device.get_buffer_memory_requirements(raw) };

        let mut allocator = ctx.allocator.lock().unwrap();
        let allocation = allocator.allocate("buffer", requirements, location)?;
        drop(allocator);

        unsafe {
            ctx.device
                .bind_buffer_memory(raw, allocation.memory(), allocation.offset())?;
        }

        Ok(Self {
            ctx,
            raw,
            allocation: Some(allocation),
            size,
            usage,
        })
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        if let Some(allocation) = self.allocation.take() {
            let mut allocator = self.ctx.allocator.lock().unwrap();
            if let Err(e) = allocator.free(allocation) {
                eprintln!("Warning: failed to free buffer allocation: {e}");
            }
        }
        unsafe { self.ctx.device.destroy_buffer(self.raw, None) };
    }
}

// ---------------------------------------------------------------------------
// Free function — one-shot GPU copy
// ---------------------------------------------------------------------------

fn copy_buffer(
    ctx: Arc<DeviceContext>,
    src: vk::Buffer,
    dst: vk::Buffer,
    size: vk::DeviceSize,
) -> anyhow::Result<()> {
    let device = &ctx.device;
    let queue = ctx.graphics_queue;
    let queue_index = ctx.graphics_queue_index;

    let pool = unsafe {
        device.create_command_pool(
            &vk::CommandPoolCreateInfo::default()
                .flags(vk::CommandPoolCreateFlags::TRANSIENT)
                .queue_family_index(queue_index),
            None,
        )?
    };

    let cmd = unsafe {
        device.allocate_command_buffers(
            &vk::CommandBufferAllocateInfo::default()
                .command_pool(pool)
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(1),
        )?[0]
    };

    unsafe {
        device.begin_command_buffer(
            cmd,
            &vk::CommandBufferBeginInfo::default()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
        )?;

        device.cmd_copy_buffer(
            cmd,
            src,
            dst,
            &[vk::BufferCopy {
                src_offset: 0,
                dst_offset: 0,
                size,
            }],
        );
        device.end_command_buffer(cmd)?;

        let fence = device.create_fence(&vk::FenceCreateInfo::default(), None)?;
        device.queue_submit(
            queue,
            &[vk::SubmitInfo::default().command_buffers(std::slice::from_ref(&cmd))],
            fence,
        )?;
        // Wait for the copy to finish before returning
        device.wait_for_fences(&[fence], true, u64::MAX)?;
        device.destroy_fence(fence, None);

        device.free_command_buffers(pool, &[cmd]);
        device.destroy_command_pool(pool, None);
    }

    Ok(())
}
