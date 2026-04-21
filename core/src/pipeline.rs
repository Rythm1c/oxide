use ash::vk;

use std::sync::Arc;

use super::context::VkContext;
use crate::vertex::Vertex;

use crate::shader::ShaderModule;

#[derive(Default, Clone)]
pub struct GraphicsPipelineConfig {
    vert_shader_path: String,
    frag_shader_path: Option<String>,
}

impl GraphicsPipelineConfig {
    pub fn vertex_shader(mut self, path: &str) -> Self {
        self.vert_shader_path = path.to_string();
        self
    }

    pub fn fragment_shader(mut self, path: &str) -> Self {
        self.frag_shader_path = Some(path.to_string());
        self
    }
}

#[derive(Clone)]
pub struct GraphicsPipeline {
    ctx: Arc<VkContext>,
    pub handle: ash::vk::Pipeline,
    pub layout: ash::vk::PipelineLayout,
    pub renderpass: ash::vk::RenderPass,
    pub framebuffers: Vec<vk::Framebuffer>,
    pub viewports: Vec<vk::Viewport>,
    pub scissors: Vec<vk::Rect2D>,
}

impl GraphicsPipeline {
    pub fn create(config: &GraphicsPipelineConfig, ctx: Arc<VkContext>) -> anyhow::Result<Self> {
        let mut pipeline = GraphicsPipeline {
            ctx,
            handle: vk::Pipeline::null(),
            layout: vk::PipelineLayout::null(),
            renderpass: vk::RenderPass::null(),
            framebuffers: Vec::new(),
            viewports: Vec::new(),
            scissors: Vec::new(),
        };

        pipeline
            .create_renderpass()
            .create_framebuffers()
            .create_layout()
            .create_pipeline(config)?;

        Ok(pipeline)
    }

    pub fn create_renderpass(&mut self) -> &mut Self {
        let renderpass_attachments = [
            ash::vk::AttachmentDescription {
                format: self.ctx.swapchain_ctx.surface_format.format,
                samples: ash::vk::SampleCountFlags::TYPE_1,
                load_op: ash::vk::AttachmentLoadOp::CLEAR,
                store_op: ash::vk::AttachmentStoreOp::STORE,
                final_layout: ash::vk::ImageLayout::PRESENT_SRC_KHR,
                ..Default::default()
            },
            ash::vk::AttachmentDescription {
                format: ash::vk::Format::D16_UNORM,
                samples: ash::vk::SampleCountFlags::TYPE_1,
                load_op: ash::vk::AttachmentLoadOp::CLEAR,
                initial_layout: ash::vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                final_layout: ash::vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                ..Default::default()
            },
        ];

        let color_attachment_ref = [ash::vk::AttachmentReference {
            attachment: 0,
            layout: ash::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        }];

        let depth_attachment_ref = ash::vk::AttachmentReference {
            attachment: 1,
            layout: ash::vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        };

        let dependencies = [ash::vk::SubpassDependency {
            src_subpass: ash::vk::SUBPASS_EXTERNAL,
            src_stage_mask: ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            dst_access_mask: ash::vk::AccessFlags::COLOR_ATTACHMENT_WRITE
                | ash::vk::AccessFlags::COLOR_ATTACHMENT_READ,
            dst_stage_mask: ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            ..Default::default()
        }];

        let subpass = ash::vk::SubpassDescription::default()
            .color_attachments(&color_attachment_ref)
            .depth_stencil_attachment(&depth_attachment_ref)
            .pipeline_bind_point(ash::vk::PipelineBindPoint::GRAPHICS);

        let renderpass_createinfo = ash::vk::RenderPassCreateInfo::default()
            .attachments(&renderpass_attachments)
            .subpasses(std::slice::from_ref(&subpass))
            .dependencies(&dependencies);

        self.renderpass = unsafe {
            self.ctx
                .device()
                .create_render_pass(&renderpass_createinfo, None)
                .expect("Failed to create render pass")
        };

        self
    }

    pub fn create_layout(&mut self) -> &mut Self {
        let layout_info = ash::vk::PipelineLayoutCreateInfo::default();

        self.layout = unsafe {
            self.ctx
                .device()
                .create_pipeline_layout(&layout_info, None)
                .expect("Failed to create pipeline layout")
        };
        self
    }

    pub fn create_framebuffers(&mut self) -> &mut Self {
        self.framebuffers = self
            .ctx
            .present_image_views()
            .iter()
            .map(|&present_image_view| {
                let framebuffer_attachments = [present_image_view, self.ctx.depth_image_view()];
                let frame_buffer_create_info = vk::FramebufferCreateInfo::default()
                    .render_pass(self.renderpass)
                    .attachments(&framebuffer_attachments)
                    .width(self.ctx.surface_resolution().width)
                    .height(self.ctx.surface_resolution().height)
                    .layers(1);

                unsafe {
                    self.ctx
                        .device()
                        .create_framebuffer(&frame_buffer_create_info, None)
                        .unwrap()
                }
            })
            .collect();

        self
    }

    pub fn create_pipeline(&mut self, cfg: &GraphicsPipelineConfig) -> anyhow::Result<Self> {
        let vertex_shader_module =
            ShaderModule::load_from_file(Arc::clone(&self.ctx.device_ctx), &cfg.vert_shader_path)?;

        let fragment_shader_module = if let Some(ref frag_path) = cfg.frag_shader_path {
            Some(ShaderModule::load_from_file(
                Arc::clone(&self.ctx.device_ctx),
                frag_path,
            )?)
        } else {
            None
        };

        let shader_entry_name = c"main";
        let shader_stage_create_infos = [
            vk::PipelineShaderStageCreateInfo {
                module: vertex_shader_module.module,
                p_name: shader_entry_name.as_ptr(),
                stage: vk::ShaderStageFlags::VERTEX,
                ..Default::default()
            },
            vk::PipelineShaderStageCreateInfo {
                module: fragment_shader_module
                    .as_ref()
                    .map(|m| m.module)
                    .unwrap_or(vk::ShaderModule::null()),
                p_name: shader_entry_name.as_ptr(),
                stage: vk::ShaderStageFlags::FRAGMENT,
                ..Default::default()
            },
        ];

        let attribute_descriptions = Vertex::get_attribute_descriptions();
        let binding_descriptions = [Vertex::get_binding_description()];
        let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_attribute_descriptions(&attribute_descriptions)
            .vertex_binding_descriptions(&binding_descriptions);

        let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo {
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            ..Default::default()
        };

        self.viewports = vec![vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: self.ctx.surface_resolution().width as f32,
            height: self.ctx.surface_resolution().height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];

        self.scissors = vec![self.ctx.surface_resolution().into()];

        let viewport_state_info = vk::PipelineViewportStateCreateInfo::default()
            .viewports(&self.viewports)
            .scissors(&self.scissors);

        let rasterization_info = vk::PipelineRasterizationStateCreateInfo {
            front_face: vk::FrontFace::COUNTER_CLOCKWISE,
            line_width: 1.0,
            polygon_mode: vk::PolygonMode::FILL,
            ..Default::default()
        };

        let multisample_state_info = vk::PipelineMultisampleStateCreateInfo {
            rasterization_samples: vk::SampleCountFlags::TYPE_1,
            ..Default::default()
        };

        let noop_stencil_state = vk::StencilOpState {
            fail_op: vk::StencilOp::KEEP,
            pass_op: vk::StencilOp::KEEP,
            depth_fail_op: vk::StencilOp::KEEP,
            compare_op: vk::CompareOp::ALWAYS,
            ..Default::default()
        };

        let depth_state_info = vk::PipelineDepthStencilStateCreateInfo {
            depth_test_enable: 1,
            depth_write_enable: 1,
            depth_compare_op: vk::CompareOp::LESS_OR_EQUAL,
            front: noop_stencil_state,
            back: noop_stencil_state,
            max_depth_bounds: 1.0,
            ..Default::default()
        };

        let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState {
            blend_enable: 0,
            src_color_blend_factor: vk::BlendFactor::SRC_COLOR,
            dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_DST_COLOR,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ZERO,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
            color_write_mask: vk::ColorComponentFlags::RGBA,
        }];

        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op(vk::LogicOp::CLEAR)
            .attachments(&color_blend_attachment_states);

        let dynamic_state = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state_info =
            vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_state);

        let graphic_pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&shader_stage_create_infos)
            .vertex_input_state(&vertex_input_state_info)
            .input_assembly_state(&vertex_input_assembly_state_info)
            .viewport_state(&viewport_state_info)
            .rasterization_state(&rasterization_info)
            .multisample_state(&multisample_state_info)
            .depth_stencil_state(&depth_state_info)
            .color_blend_state(&color_blend_state)
            .dynamic_state(&dynamic_state_info)
            .layout(self.layout)
            .render_pass(self.renderpass);

        self.handle = unsafe {
            self.ctx
                .device()
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &[graphic_pipeline_info],
                    None,
                )
                .expect("Unable to create graphics pipeline")[0]
        };

        Ok(self.clone())
    }
}

impl Drop for GraphicsPipeline {
    fn drop(&mut self) {
        // We can't destroy the pipeline here because we need the context, and we don't have it.
        // The pipeline should be explicitly destroyed by the owner of the context before the context is dropped.
        unsafe {
            for &framebuffer in &self.framebuffers {
                self.ctx.device().destroy_framebuffer(framebuffer, None);
            }
            self.ctx.device().destroy_render_pass(self.renderpass, None);
            self.ctx.device().destroy_pipeline_layout(self.layout, None);
            self.ctx.device().destroy_pipeline(self.handle, None);
        }
    }
}
