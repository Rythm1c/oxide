use super::context::{VkContext, record_submit_commandbuffer};
use ash::vk;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Scene / RenderObject
// ---------------------------------------------------------------------------

pub struct RenderObject {
    pub vertex_buffer: vk::Buffer,
    pub index_buffer: Option<vk::Buffer>,
    pub vertex_count: u32,
    pub index_count: u32,
}

pub struct Scene {
    objects: Vec<RenderObject>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    pub fn add_object(&mut self, object: RenderObject) {
        self.objects.push(object);
    }

    pub fn objects(&self) -> &[RenderObject] {
        &self.objects
    }
}

// ---------------------------------------------------------------------------
// Renderer
// ---------------------------------------------------------------------------

const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub struct Renderer {
    context: Arc<VkContext>,
    current_scene: Option<Scene>,
    clear_color: [f32; 4],

    draw_command_buffers: Vec<vk::CommandBuffer>,

    // Per-frame sync
    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,
    current_frame: usize,
}

impl Renderer {
    pub fn new(context: Arc<VkContext>) -> Self {
        let device = context.device();

        let semaphore_info = vk::SemaphoreCreateInfo::default();
        // Fences start signalled so the first wait in `render` doesn't block
        let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);

        let draw_command_buffers = unsafe {
            device
                .allocate_command_buffers(
                    &vk::CommandBufferAllocateInfo::default()
                        .command_buffer_count(MAX_FRAMES_IN_FLIGHT as u32)
                        .command_pool(context.device_ctx.pool)
                        .level(vk::CommandBufferLevel::PRIMARY),
                )
                .expect("Failed to allocate draw command buffers")
        };

        let mut image_available_semaphores = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let mut render_finished_semaphores = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let mut in_flight_fences = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);

        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            unsafe {
                image_available_semaphores
                    .push(device.create_semaphore(&semaphore_info, None).unwrap());
                render_finished_semaphores
                    .push(device.create_semaphore(&semaphore_info, None).unwrap());
                in_flight_fences.push(device.create_fence(&fence_info, None).unwrap());
            }
        }

        Self {
            context,
            current_scene: None,
            clear_color: [0.1, 0.1, 0.15, 1.0],
            draw_command_buffers,
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            current_frame: 0,
        }
    }

    pub fn set_scene(&mut self, scene: Scene) {
        self.current_scene = Some(scene);
    }

    pub fn set_clear_color(&mut self, color: [f32; 4]) {
        self.clear_color = color;
    }

    /// Draw one frame.
    ///
    /// Fix applied: `reset_fences` now happens inside `record_submit_commandbuffer`
    /// (which already calls wait → reset → begin → record → end → submit).
    /// The redundant manual wait+reset in the old `render` is removed.
    pub fn render(&mut self, pipeline: &super::pipeline::GraphicsPipeline) -> anyhow::Result<()> {
        let ctx = &self.context;
        let device = ctx.device();
        let frame = self.current_frame;

        let cmd = self.draw_command_buffers[frame];
        let image_available = self.image_available_semaphores[frame];
        let render_finished = self.render_finished_semaphores[frame];
        let fence = self.in_flight_fences[frame];

        // Acquire the next swapchain image.
        // We pass `image_available` as the semaphore to signal.
        // The fence here is null — we use our own per-frame fence via
        // record_submit_commandbuffer which handles wait+reset correctly.
        let (present_index, _suboptimal) = unsafe {
            ctx.swapchain_loader().acquire_next_image(
                ctx.swapchain(),
                u64::MAX,
                image_available,
                vk::Fence::null(),
            )?
        };

        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: self.clear_color,
                },
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            },
        ];

        let render_pass_begin_info = vk::RenderPassBeginInfo::default()
            .render_pass(pipeline.renderpass)
            .framebuffer(pipeline.framebuffers[present_index as usize])
            .render_area(ctx.surface_resolution().into())
            .clear_values(&clear_values);

        // record_submit_commandbuffer:
        //   1. wait_for_fences(fence)   ← blocks until previous frame done
        //   2. reset_fences(fence)      ← ✓ fence is now unsignalled
        //   3. begin / record / end
        //   4. queue_submit(..., fence) ← GPU will signal fence when done
        record_submit_commandbuffer(
            device,
            cmd,
            fence,
            ctx.present_queue(),
            &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT],
            &[image_available],
            &[render_finished],
            |device, cmd| unsafe {
                device.cmd_begin_render_pass(
                    cmd,
                    &render_pass_begin_info,
                    vk::SubpassContents::INLINE,
                );
                device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, pipeline.handle);
                device.cmd_set_viewport(cmd, 0, &pipeline.viewports);
                device.cmd_set_scissor(cmd, 0, &pipeline.scissors);

                if let Some(scene) = &self.current_scene {
                    for obj in scene.objects() {
                        device.cmd_bind_vertex_buffers(cmd, 0, &[obj.vertex_buffer], &[0]);

                        if let Some(ib) = obj.index_buffer {
                            device.cmd_bind_index_buffer(cmd, ib, 0, vk::IndexType::UINT32);
                            device.cmd_draw_indexed(cmd, obj.index_count, 1, 0, 0, 0);
                        } else {
                            device.cmd_draw(cmd, obj.vertex_count, 1, 0, 0);
                        }
                    }
                }

                device.cmd_end_render_pass(cmd);
            },
        );

        // Present
        unsafe {
            ctx.swapchain_loader().queue_present(
                ctx.present_queue(),
                &vk::PresentInfoKHR::default()
                    .wait_semaphores(&[render_finished])
                    .swapchains(&[ctx.swapchain()])
                    .image_indices(&[present_index]),
            )?;
        }

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
        Ok(())
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        let device = self.context.device();
        unsafe {
            // Wait on all fences so we know the GPU is idle
            device
                .wait_for_fences(&self.in_flight_fences, true, u64::MAX)
                .unwrap();

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
