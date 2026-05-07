use super::context::{VkContext, record_submit_commandbuffer_no_wait};
use super::descriptor::GlobalDescriptorSet;
use super::pipeline::RenderPipeline;

use crate::drawable::{RenderObject, render_drawable};
use crate::pipeline::PushConstants;
use crate::shadowmap::ShadowMap;

use ash::vk;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Renderer
// ---------------------------------------------------------------------------
pub struct Renderer {
    context: Arc<VkContext>,
    clear_color: [f32; 4],

    draw_command_buffers: Vec<vk::CommandBuffer>,
    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,
    current_frame: usize,
}

impl Renderer {
    pub const MAX_FRAMES_IN_FLIGHT: usize = 2;

    pub fn new(context: Arc<VkContext>) -> Self {
        let device = context.device();

        let draw_command_buffers = unsafe {
            device
                .allocate_command_buffers(
                    &vk::CommandBufferAllocateInfo::default()
                        .command_buffer_count(Self::MAX_FRAMES_IN_FLIGHT as u32)
                        .command_pool(context.device_ctx.pool)
                        .level(vk::CommandBufferLevel::PRIMARY),
                )
                .expect("Failed to allocate draw command buffers")
        };

        let semaphore_info = vk::SemaphoreCreateInfo::default();
        // Start fences signalled so the first frame doesn't wait forever
        let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);

        let swapchain_image_count = context.present_image_views().len();
        let mut image_available_semaphores = Vec::with_capacity(Self::MAX_FRAMES_IN_FLIGHT);
        let mut render_finished_semaphores = Vec::with_capacity(swapchain_image_count);
        let mut in_flight_fences = Vec::with_capacity(Self::MAX_FRAMES_IN_FLIGHT);

        for _ in 0..Self::MAX_FRAMES_IN_FLIGHT {
            unsafe {
                image_available_semaphores
                    .push(device.create_semaphore(&semaphore_info, None).unwrap());
                in_flight_fences.push(device.create_fence(&fence_info, None).unwrap());
            }
        }

        for _ in 0..swapchain_image_count {
            unsafe {
                render_finished_semaphores
                    .push(device.create_semaphore(&semaphore_info, None).unwrap());
            }
        }

        Self {
            context,
            clear_color: [0.0, 0.0, 0.0, 1.0],
            draw_command_buffers,
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            current_frame: 0,
        }
    }

    pub fn set_clear_color(&mut self, color: [f32; 4]) {
        self.clear_color = color;
    }

    pub fn get_current_frame(&self) -> usize {
        self.current_frame
    }
    /// Render one frame.
    ///
    /// Per-object `PushConstants` are uploaded via `cmd_push_constants` for
    /// every draw call, so the shader sees each object's correct MVP matrix.
    pub fn render(
        &mut self,
        pipeline: &RenderPipeline,
        globals: &GlobalDescriptorSet,
        shadow_map: &ShadowMap,
        light_space_matrix: [[f32; 4]; 4],
        render_objects: &Vec<RenderObject>,
    ) -> anyhow::Result<()> {
        let ctx = &self.context;
        let device = ctx.device();
        let frame = self.current_frame;

        let cmd = self.draw_command_buffers[frame];
        let image_available = self.image_available_semaphores[frame];
        let fence = self.in_flight_fences[frame];

        // IMPORTANT: Wait for this frame's fence BEFORE reusing its semaphores.
        // This ensures the previous use of image_available semaphore has been fully consumed.
        unsafe {
            device.wait_for_fences(&[fence], true, u64::MAX)?;
            device.reset_fences(&[fence])?;
        }

        // Now safe to acquire next image — previous frame's submission is complete
        let (present_index, _suboptimal) = unsafe {
            ctx.swapchain_loader().acquire_next_image(
                ctx.swapchain(),
                u64::MAX,
                image_available,
                vk::Fence::null(),
            )?
        };

        // Use one render-finished semaphore per swapchain image.
        // The semaphore that signals rendering completion must not be reused
        // until the corresponding present operation on that image has finished.
        let render_finished = self.render_finished_semaphores[present_index as usize];

        // record_submit_commandbuffer_no_wait:
        //   1. Skip wait/reset (we already did it above)
        //   2. begin / record / end
        //   3. queue_submit(..., fence) — GPU will signal fence when done
        record_submit_commandbuffer_no_wait(
            device,
            cmd,
            fence,
            ctx.present_queue(),
            &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT],
            &[image_available],
            &[render_finished],
            |device, cmd| unsafe {
                // render objects to the shadow map
                {
                    shadow_map.begin_render_pass(cmd);

                    for obj in render_objects.iter() {
                        shadow_map.render_shadow(cmd, obj, light_space_matrix);
                    }
                    // end render pipeline's render pass
                    device.cmd_end_render_pass(cmd);
                }

                // render objects normally
                {
                    pipeline.begin_render_pass(cmd, self.clear_color, present_index as usize);

                    device.cmd_bind_descriptor_sets(
                        cmd,
                        vk::PipelineBindPoint::GRAPHICS,
                        pipeline.layout,
                        0,
                        &[globals.set(frame)],
                        &[],
                    );

                    for obj in render_objects.iter() {
                        // Upload this object's MVP + model matrix as push constants.
                        // This is the cheapest way to give each physics body its
                        // own transform — no UBO allocation needed.
                        device.cmd_push_constants(
                            cmd,
                            pipeline.layout,
                            vk::ShaderStageFlags::VERTEX,
                            0,
                            PushConstants::from_model_matrix(obj.model_matrix).as_bytes(),
                        );

                        device.cmd_bind_descriptor_sets(
                            cmd,
                            vk::PipelineBindPoint::GRAPHICS,
                            pipeline.layout,
                            1,
                            &[obj.material_desc.set],
                            &[],
                        );

                        render_drawable(device, cmd, obj);
                    }
                    // end render pipeline's render pass
                    device.cmd_end_render_pass(cmd);
                }
            },
        );

        unsafe {
            ctx.swapchain_loader().queue_present(
                ctx.present_queue(),
                &vk::PresentInfoKHR::default()
                    .wait_semaphores(&[render_finished])
                    .swapchains(&[ctx.swapchain()])
                    .image_indices(&[present_index]),
            )?;
        }

        self.current_frame = (self.current_frame + 1) % Self::MAX_FRAMES_IN_FLIGHT;
        Ok(())
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        let device = self.context.device();
        unsafe {
            // Wait for all in-flight frames before destroying sync objects
            if !self.in_flight_fences.is_empty() {
                device
                    .wait_for_fences(&self.in_flight_fences, true, u64::MAX)
                    .unwrap();
            }
            for s in self.image_available_semaphores.drain(..) {
                device.destroy_semaphore(s, None);
            }
            for s in self.render_finished_semaphores.drain(..) {
                device.destroy_semaphore(s, None);
            }
            for f in self.in_flight_fences.drain(..) {
                device.destroy_fence(f, None);
            }
        }
    }
}
