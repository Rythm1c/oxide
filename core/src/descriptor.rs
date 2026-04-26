use ash::vk;
use std::sync::Arc;

use crate::ubo::MaterialUbo;

use super::buffer::{Buffer, BufferUsage};
use super::device::DeviceContext;
use super::ubo::{CameraUbo, LightUbo};

// global set for the camera and light UBOs, which are shared across all draw calls
pub struct GlobalDescriptorSet {
    ctx           : Arc<DeviceContext>,

    layout        : vk::DescriptorSetLayout,
    pool          : vk::DescriptorPool,
    sets          : Vec<vk::DescriptorSet>, // one per frame in flight

    camera_buffers: Vec<Buffer>,
    light_buffers : Vec<Buffer>,
}

impl GlobalDescriptorSet {
    pub fn new(ctx: Arc<DeviceContext>, frames_in_flight: usize) -> anyhow::Result<Self> {
        let device = &ctx.device;

        let bindings = [
            vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::VERTEX),
            vk::DescriptorSetLayoutBinding::default()
                .binding(1)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT),
        ];

        let layout = unsafe {
            device.create_descriptor_set_layout(
                &vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings),
                None,
            )?
        };

        // We need `frames_in_flight` sets, each with 2 UBO descriptors.
        let pool_sizes = [vk::DescriptorPoolSize {
            ty: vk::DescriptorType::UNIFORM_BUFFER,
            // 2 bindings × frames_in_flight
            descriptor_count: (2 * frames_in_flight) as u32,
        }];

        let pool = unsafe {
            device.create_descriptor_pool(
                &vk::DescriptorPoolCreateInfo::default()
                    .max_sets(frames_in_flight as u32)
                    .pool_sizes(&pool_sizes),
                None,
            )?
        };

        let layouts: Vec<vk::DescriptorSetLayout> = vec![layout; frames_in_flight];
        let sets = unsafe {
            device.allocate_descriptor_sets(
                &vk::DescriptorSetAllocateInfo::default()
                    .descriptor_pool(pool)
                    .set_layouts(&layouts),
            )?
        };

        let camera_size = std::mem::size_of::<CameraUbo>() as vk::DeviceSize;
        let light_size = std::mem::size_of::<LightUbo>() as vk::DeviceSize;

        let mut camera_buffers = Vec::with_capacity(frames_in_flight);
        let mut light_buffers = Vec::with_capacity(frames_in_flight);

        for _ in 0..frames_in_flight {
            camera_buffers.push(Buffer::new(
                Arc::clone(&ctx),
                camera_size,
                BufferUsage::UNIFORM,
            )?);
            light_buffers.push(Buffer::new(
                Arc::clone(&ctx),
                light_size,
                BufferUsage::UNIFORM,
            )?);
        }

        let mut descriptor_writes = Vec::with_capacity(frames_in_flight * 2);

        let mut camera_infos: Vec<[vk::DescriptorBufferInfo; 1]> =
            Vec::with_capacity(frames_in_flight);
        let mut light_infos: Vec<[vk::DescriptorBufferInfo; 1]> =
            Vec::with_capacity(frames_in_flight);

        for i in 0..frames_in_flight {
            camera_infos.push([vk::DescriptorBufferInfo {
                buffer: camera_buffers[i].raw,
                offset: 0,
                range: camera_size,
            }]);
            light_infos.push([vk::DescriptorBufferInfo {
                buffer: light_buffers[i].raw,
                offset: 0,
                range: light_size,
            }]);
        }

        for i in 0..frames_in_flight {
            descriptor_writes.push(
                vk::WriteDescriptorSet::default()
                    .dst_set(sets[i])
                    .dst_binding(0)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .buffer_info(&camera_infos[i]),
            );
            descriptor_writes.push(
                vk::WriteDescriptorSet::default()
                    .dst_set(sets[i])
                    .dst_binding(1)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .buffer_info(&light_infos[i]),
            );
        }

        unsafe { device.update_descriptor_sets(&descriptor_writes, &[]) };

        Ok(Self {
            ctx,
            layout,
            pool,
            camera_buffers,
            light_buffers,
            sets,
        })
    }

    pub fn layout(&self) -> vk::DescriptorSetLayout {
        self.layout
    }

    pub fn set(&self, frame: usize) -> vk::DescriptorSet {
        self.sets[frame]
    }

    pub fn flush(
        &mut self,
        frame : usize,
        camera: &CameraUbo,
        light : &LightUbo,
    ) -> anyhow::Result<()> {
        self.camera_buffers[frame].write(std::slice::from_ref(camera))?;
        self.light_buffers[frame].write(std::slice::from_ref(light))?;
        Ok(())
    }
}

impl Drop for GlobalDescriptorSet {
    fn drop(&mut self) {
        self.camera_buffers.clear();
        self.light_buffers.clear();

        unsafe {
            self.ctx.device.destroy_descriptor_pool(self.pool, None);
            self.ctx
                .device
                .destroy_descriptor_set_layout(self.layout, None);
        }
    }
}

pub struct MaterialAllocator {
    ctx      : Arc<DeviceContext>,
    layout   : vk::DescriptorSetLayout,
    pool     : vk::DescriptorPool,
    capacity : u32,
    allocated: u32,
}

impl MaterialAllocator {
    pub fn new(ctx: Arc<DeviceContext>, capacity: u32) -> anyhow::Result<Self> {
        let device = &ctx.device;

        let bindings = [vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)];

        let layout = unsafe {
            device.create_descriptor_set_layout(
                &vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings),
                None,
            )?
        };

        // One UBO descriptor per set, `capacity` sets total.
        let pool_sizes = [vk::DescriptorPoolSize {
            ty: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: capacity,
        }];

        let pool = unsafe {
            device.create_descriptor_pool(
                &vk::DescriptorPoolCreateInfo::default()
                    .max_sets(capacity)
                    .pool_sizes(&pool_sizes),
                None,
            )?
        };

        Ok(Self {
            ctx,
            layout,
            pool,
            capacity,
            allocated: 0,
        })
    }

    pub fn allocate(&mut self, initial: &MaterialUbo) -> anyhow::Result<MaterialDescriptorSet> {
        if self.allocated >= self.capacity {
            anyhow::bail!(
                "MaterialAllocator pool exhausted ({} / {}). \
                Recreate with a larger capacity.",
                self.allocated,
                self.capacity
            );
        }

        let set_layouts = [self.layout];
        let sets = unsafe {
            self.ctx.device.allocate_descriptor_sets(
                &vk::DescriptorSetAllocateInfo::default()
                    .descriptor_pool(self.pool)
                    .set_layouts(&set_layouts),
            )?
        };

        let material_size = std::mem::size_of::<MaterialUbo>() as vk::DeviceSize;
        let mut buffer = Buffer::new(Arc::clone(&self.ctx), material_size, BufferUsage::UNIFORM)?;
        buffer.write(std::slice::from_ref(initial))?;

        let buffer_info = [vk::DescriptorBufferInfo::default()
            .buffer(buffer.raw)
            .offset(0)
            .range(material_size)];

        let write = vk::WriteDescriptorSet::default()
            .dst_set(sets[0])
            .dst_binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(&buffer_info);

        unsafe { self.ctx.device.update_descriptor_sets(&[write], &[]) };

        self.allocated += 1;

        Ok(MaterialDescriptorSet {
            ctx: Arc::clone(&self.ctx),
            set: sets[0],
            buffer,
        })
    }

    pub fn layout(&self) -> vk::DescriptorSetLayout {
        self.layout
    }
    pub fn allocated(&self) -> u32 {
        self.allocated
    }
    pub fn capacity(&self) -> u32 {
        self.capacity
    }
}

impl Drop for MaterialAllocator {
    fn drop(&mut self) {
        unsafe {
            self.ctx.device.destroy_descriptor_pool(self.pool, None);
            self.ctx
                .device
                .destroy_descriptor_set_layout(self.layout, None);
        }
    }
}

pub struct MaterialDescriptorSet {
    ctx    : Arc<DeviceContext>,
    pub set: vk::DescriptorSet,
    buffer : Buffer,
}

impl MaterialDescriptorSet {
    /// Update material properties (call when material changes, not every frame).
    pub fn flush(&mut self, material: &MaterialUbo) -> anyhow::Result<()> {
        self.buffer.write(std::slice::from_ref(material))
    }
}

impl Drop for MaterialDescriptorSet {
    fn drop(&mut self) {
    }
}
