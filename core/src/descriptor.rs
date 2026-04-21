use ash::vk;

use super::buffer::Buffer;
use super::ubo::{CameraUBO, LightUBO};
use super::device::DeviceContext;

// global set for the camera and light UBOs, which are shared across all draw calls
pub struct GlobalDescriptorSet {
    ctx: DeviceContext,

    pub set_layout: vk::DescriptorSetLayout,
    pub pool: vk::DescriptorPool,
    pub set: Vec<vk::DescriptorSet>,// one per frame in flight

    pub camera_buffer: Vec<Buffer>,
    pub light_buffer: Vec<Buffer>,
}

impl GlobalDescriptorSet {
    pub fn new(ctx: DeviceContext) -> anyhow::Result<Self> {
        // Create descriptor set layout
        let set_layout = unsafe {
            ctx.device.create_descriptor_set_layout(
                &vk::DescriptorSetLayoutCreateInfo::default().bindings(&[
                    vk::DescriptorSetLayoutBinding {
                        binding: 0,
                        descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                        descriptor_count: 1,
                        stage_flags: vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                        ..Default::default()
                    },
                    vk::DescriptorSetLayoutBinding {
                        binding: 1,
                        descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                        descriptor_count: 1,
                        stage_flags: vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                        ..Default::default()
                    },
                ]),
                None,
            )
        }?;

        // Create descriptor pool
        let pool = unsafe {
            ctx.device.create_descriptor_pool(
                &vk::DescriptorPoolCreateInfo::default()
                    .pool_sizes(&[
                        vk::DescriptorPoolSize {
                            ty: vk::DescriptorType::UNIFORM_BUFFER,
                            descriptor_count: (2 * MAX_FRAMES_IN_FLIGHT) as u32, // camera + light per frame
                        },
                    ])
                    .max_sets(MAX_FRAMES_IN_FLIGHT as u32),
                None,
            )
        }?;

        // Allocate descriptor sets
        let set_layouts = vec![set_layout; MAX_FRAMES_IN_FLIGHT];
        let sets = unsafe {
            ctx.device.allocate_descriptor_sets(
                &vk::DescriptorSetAllocateInfo::default()
                    .descriptor_pool(pool)
                    .set_layouts(&set_layouts),
            )
        }?;

        // Create buffers for each frame in flight
        let mut camera_buffers = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let mut light_buffers = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);

        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            camera_buffers.push(Buffer::new_uniform_buffer(
                &ctx,
                std::mem::size_of::<CameraUBO>() as u64,
            )?);
            light_buffers.push(Buffer::new_uniform_buffer(
                &ctx,
                std::mem::size_of::<LightUBO>() as u64,
            )?);
        }

        // Update descriptor sets to point to
        for i in 0..MAX_FRAMES_IN_FLIGHT {
            let camera_buffer_info = vk::DescriptorBufferInfo {
                buffer: camera_buffers[i].buffer,
                offset: 0,
                range: std::mem::size_of::<CameraUBO>() as u64,
            };
            let light_buffer_info = vk::DescriptorBufferInfo {
                buffer: light_buffers[i].buffer,
                offset: 0,
                range: std::mem::size_of::<LightUBO>() as u64,
            };

            let descriptor_writes = [
                vk::WriteDescriptorSet {
                    dst_set: sets[i],
                    dst_binding: 0,
                    descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                    buffer_info: std::slice::from_ref(&camera_buffer_info),
                    ..Default::default()
                },
                vk::WriteDescriptorSet {
                    dst_set: sets[i],
                    dst_binding: 1,
                    descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                    buffer_info: std::slice::from_ref(&light_buffer_info),
                    ..Default::default()
                },
            ];

            unsafe {
                ctx.device.update_descriptor_sets(&descriptor_writes, &[]);
            }
        }
    }
    pub fn update(&mut self, camera_ubo: &CameraUBO, light_ubo: &LightUBO) -> anyhow::Result<()> {
        self.camera_buffer.write(&[camera_ubo])?;
        self.light_buffer.write(&[light_ubo])?;
        Ok(())
    }
}

impl Drop for GlobalDescriptorSet{
    fn drop(&mut self) {
        unsafe {
            self.ctx.device.destroy_descriptor_pool(self.pool, None);
            self.ctx.device.destroy_descriptor_set_layout(self.set_layout, None);
    }
    }
}
