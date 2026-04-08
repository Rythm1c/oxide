use super::context::VkContext;
use ash::vk;
use std::{sync::Arc, u64};

pub struct RenderObject {
    pub vertex_buffer: vk::Buffer,
    pub index_buffer: Option<vk::Buffer>,
    pub index_count: u32,
    pub vertex_count: u32,
}

pub struct Scene {
    objects: Vec<RenderObject>,
}

impl Scene {
    pub fn new() -> Self {
        Scene {
            objects: Vec::new(),
        }
    }

    pub fn add_object(&mut self, object: RenderObject) {
        self.objects.push(object);
    }
}

const MAX_FRAMES_IN_FLIGHT: usize = 2;
pub struct Renderer {
    context: Arc<VkContext>,
    current_scene: Option<Scene>,
    clear_color: [f32; 4],

    draw_command_buffers: Vec<vk::CommandBuffer>,
    //sync variables
    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,
    current_frame: usize,
}

impl Renderer {
    pub fn new(context: Arc<VkContext>) -> Self {
        let device = &context.device;
        let mut image_available_semaphores = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let mut render_finished_semaphores = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let mut in_flight_fences = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);

        let semaphore_info = vk::SemaphoreCreateInfo::default();
        let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);

        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::default()
            .command_buffer_count(MAX_FRAMES_IN_FLIGHT as u32)
            .command_pool(context.pool)
            .level(vk::CommandBufferLevel::PRIMARY);

        let draw_command_buffers = unsafe {
            device
                .allocate_command_buffers(&command_buffer_allocate_info)
                .unwrap()
        };

        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            image_available_semaphores
                .push(unsafe { device.create_semaphore(&semaphore_info, None).unwrap() });
            render_finished_semaphores
                .push(unsafe { device.create_semaphore(&semaphore_info, None).unwrap() });
            in_flight_fences.push(unsafe { device.create_fence(&fence_info, None).unwrap() });
        }
        Self {
            context,
            current_scene: None,
            clear_color: [0.2, 0.2, 0.2, 1.0],
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

    pub fn render(
        &mut self,
        pipeline: &super::pipeline::GraphicsPipeline,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let base = &self.context;
        let device = &base.device;
        let command_buffer = self.draw_command_buffers[self.current_frame];
        let image_available = self.image_available_semaphores[self.current_frame];
        let render_finished = self.render_finished_semaphores[self.current_frame];
        let in_flight_fence = self.in_flight_fences[self.current_frame];

        unsafe {
            device
                .wait_for_fences(&[in_flight_fence], true, u64::MAX)
                .unwrap();
            //device.reset_fences(&[in_flight_fence]).unwrap();
        }

        let (present_index, _) = unsafe {
            base.swapchain_loader
                .acquire_next_image(base.swapchain, u64::MAX, image_available, vk::Fence::null())
                .unwrap()
        };

        unsafe {
            device
                .reset_command_buffer(
                    command_buffer,
                    vk::CommandBufferResetFlags::RELEASE_RESOURCES,
                )
                .expect("Reset command buffer failed.");
        }

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
            .render_area(base.surface_resolution.into())
            .clear_values(&clear_values);

        super::context::record_submit_commandbuffer(
            &base.device,
            command_buffer,
            in_flight_fence,
            base.present_queue,
            &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT],
            &[image_available],
            &[render_finished],
            |device, draw_command_buffer| {
                unsafe {
                    device.cmd_begin_render_pass(
                        draw_command_buffer,
                        &render_pass_begin_info,
                        vk::SubpassContents::INLINE,
                    );
                    device.cmd_bind_pipeline(
                        draw_command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        pipeline.handle,
                    );
                    device.cmd_set_viewport(draw_command_buffer, 0, &pipeline.viewports);
                    device.cmd_set_scissor(draw_command_buffer, 0, &pipeline.scissors);

                    // Render all objects in the scene
                    if let Some(scene) = &self.current_scene {
                        for object in &scene.objects {
                            device.cmd_bind_vertex_buffers(
                                draw_command_buffer,
                                0,
                                &[object.vertex_buffer],
                                &[0],
                            );

                            if let Some(index_buffer) = object.index_buffer {
                                device.cmd_bind_index_buffer(
                                    draw_command_buffer,
                                    index_buffer,
                                    0,
                                    vk::IndexType::UINT32,
                                );
                                device.cmd_draw_indexed(
                                    draw_command_buffer,
                                    object.index_count,
                                    1,
                                    0,
                                    0,
                                    1,
                                );
                            } else {
                                device.cmd_draw(draw_command_buffer, object.vertex_count, 1, 0, 0);
                            }
                        }
                    }

                    device.cmd_end_render_pass(draw_command_buffer);
                }
            },
        );

        let wait_semaphores = [render_finished];
        let swapchains = [base.swapchain];
        let image_indices = [present_index];
        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(&wait_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        unsafe {
            base.swapchain_loader
                .queue_present(base.present_queue, &present_info)
                .unwrap()
        };

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;

        Ok(())
    }

    pub fn destroy(&self, context: Arc<VkContext>) {
        unsafe {
            for present_complete_semaphore in &self.image_available_semaphores {
                context
                    .device
                    .destroy_semaphore(*present_complete_semaphore, None);
            }

            for rendering_complete_semaphore in &self.render_finished_semaphores {
                context
                    .device
                    .destroy_semaphore(*rendering_complete_semaphore, None);
            }

            for draw_commands_reuse_fence in &self.in_flight_fences {
                context
                    .device
                    .destroy_fence(*draw_commands_reuse_fence, None);
            }
        }
    }
}
