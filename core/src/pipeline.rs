use ash::util::read_spv;
use ash::vk;

use crate::vertex::Vertex;

use super::context::VkContext;

#[derive(Debug, Default, Clone)]
pub struct GraphicsPipeline {
    pub handle: ash::vk::Pipeline,
    pub layout: ash::vk::PipelineLayout,
    pub renderpass: ash::vk::RenderPass,
    pub framebuffers: Vec<vk::Framebuffer>,
    pub viewports: Vec<vk::Viewport>,
    pub scissors: Vec<vk::Rect2D>,
    fragment_shader_module: ash::vk::ShaderModule,
    vertex_shader_module: ash::vk::ShaderModule,
}

impl GraphicsPipeline {
    pub fn create_renderpass(&mut self, context: &VkContext) -> &mut Self {
        let renderpass_attachments = [
            ash::vk::AttachmentDescription {
                format: context.surface_format.format,
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
            context
                .device
                .create_render_pass(&renderpass_createinfo, None)
                .expect("Failed to create render pass")
        };

        self
    }

    pub fn create_layout(&mut self, context: &VkContext) -> &mut Self {
        let layout_info = ash::vk::PipelineLayoutCreateInfo::default();

        self.layout = unsafe {
            context
                .device
                .create_pipeline_layout(&layout_info, None)
                .expect("Failed to create pipeline layout")
        };
        self
    }

    pub fn create_framebuffers(&mut self, context: &VkContext) -> &mut Self {
        self.framebuffers = context
            .present_image_views
            .iter()
            .map(|&present_image_view| {
                let framebuffer_attachments = [present_image_view, context.depth_image_view];
                let frame_buffer_create_info = vk::FramebufferCreateInfo::default()
                    .render_pass(self.renderpass)
                    .attachments(&framebuffer_attachments)
                    .width(context.surface_resolution.width)
                    .height(context.surface_resolution.height)
                    .layers(1);

                unsafe {
                    context
                        .device
                        .create_framebuffer(&frame_buffer_create_info, None)
                        .unwrap()
                }
            })
            .collect();

        self
    }

    pub fn create_shader_modules(
        &mut self,
        context: &VkContext,
        vert_path: &str,
        frag_path: &str,
    ) -> &mut Self {
        let vert_code = read_shader_spv(vert_path).unwrap();
        let vertex_shader_info = vk::ShaderModuleCreateInfo::default().code(&vert_code);

        let frag_code = read_shader_spv(frag_path).unwrap();
        let fragment_shader_info = vk::ShaderModuleCreateInfo::default().code(&frag_code);
        self.vertex_shader_module = unsafe {
            context
                .device
                .create_shader_module(&vertex_shader_info, None)
                .expect("Failed to create vertex shader module")
        };
        self.fragment_shader_module = unsafe {
            context
                .device
                .create_shader_module(&fragment_shader_info, None)
                .expect("Failed to create fragment shader module")
        };

        self
    }

    pub fn build(&mut self, context: &VkContext) -> Result<Self, std::io::Error> {
        let shader_entry_name = c"main";
        let shader_stage_create_infos = [
            vk::PipelineShaderStageCreateInfo {
                module: self.vertex_shader_module,
                p_name: shader_entry_name.as_ptr(),
                stage: vk::ShaderStageFlags::VERTEX,
                ..Default::default()
            },
            vk::PipelineShaderStageCreateInfo {
                module: self.fragment_shader_module,
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
            width: context.surface_resolution.width as f32,
            height: context.surface_resolution.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];

        self.scissors = vec![context.surface_resolution.into()];

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
            context
                .device
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &[graphic_pipeline_info],
                    None,
                )
                .expect("Unable to create graphics pipeline")[0]
        };

        Ok(self.clone())
    }

    pub fn destroy(&mut self, context: &VkContext) {
        unsafe {
            context
                .device
                .destroy_shader_module(self.vertex_shader_module, None);
            context
                .device
                .destroy_shader_module(self.fragment_shader_module, None);
            for &framebuffer in &self.framebuffers {
                context.device.destroy_framebuffer(framebuffer, None);
            }
            context.device.destroy_render_pass(self.renderpass, None);
            context.device.destroy_pipeline_layout(self.layout, None);
            context.device.destroy_pipeline(self.handle, None);
        }
    }
}

use std::fs::File;
use std::io::BufReader;

// Reads a SPIR-V file and returns a Vec<u32> containing the binary code
fn read_shader_spv(path: &str) -> std::io::Result<Vec<u32>> {
    // Open the file in read-only mode
    let file = File::open(path)?;
    // Wrap the file in a buffered reader for efficient reading
    let mut reader = BufReader::new(file);
    // Use ash's utility to read the SPIR-V binary into a Vec<u32>
    let spv = read_spv(&mut reader)?;
    Ok(spv)
}
