use ash::vk;
use std::sync::Arc;

use super::context::VkContext;
use crate::vertex::Vertex;
use crate::shader::ShaderModule;

// ---------------------------------------------------------------------------
// Push constants — 128 bytes guaranteed by the Vulkan spec.
// Holds model, view, and projection matrices (3 × 16 floats × 4 bytes = 192).
// Since that's over 128 bytes we pack MVP as a combined model-view-projection
// matrix (1 × 16 floats = 64 bytes) plus the model matrix for lighting (64).
//
// Layout (std430):
//   offset  0: mat4 mvp      (model * view * proj, pre-multiplied on CPU)
//   offset 64: mat4 model    (for normal/lighting transforms in the shader)
//
// In the vertex shader use:
//   layout(push_constant) uniform PushConstants {
//       mat4 mvp;
//       mat4 model;
//   } pc;
// ---------------------------------------------------------------------------
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PushConstants {
    pub mvp: [[f32; 4]; 4],   // 64 bytes — pre-multiplied MVP
    pub model: [[f32; 4]; 4], // 64 bytes — model matrix for normals/lighting
}

impl PushConstants {
    pub fn identity() -> Self {
        let id = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        Self { mvp: id, model: id }
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self as *const Self as *const u8,
                std::mem::size_of::<Self>(),
            )
        }
    }
}

// ---------------------------------------------------------------------------
// Config — builder style
// ---------------------------------------------------------------------------

#[derive(Clone, Default)]
pub struct GraphicsPipelineConfig {
    vert_shader_path: String,
    frag_shader_path: Option<String>,
    pub polygon_mode: Option<vk::PolygonMode>,
    pub cull_mode: Option<vk::CullModeFlags>,
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

    /// Override rasterisation fill mode (default: FILL).
    /// Use `vk::PolygonMode::LINE` for wireframe debug rendering.
    pub fn polygon_mode(mut self, mode: vk::PolygonMode) -> Self {
        self.polygon_mode = Some(mode);
        self
    }

    /// Override back-face culling (default: BACK).
    pub fn cull_mode(mut self, mode: vk::CullModeFlags) -> Self {
        self.cull_mode = Some(mode);
        self
    }
}

// ---------------------------------------------------------------------------
// GraphicsPipeline
// ---------------------------------------------------------------------------

pub struct GraphicsPipeline {
    ctx: Arc<VkContext>,
    pub handle: vk::Pipeline,
    pub layout: vk::PipelineLayout,
    pub renderpass: vk::RenderPass,
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
            .create_layout()
            .create_framebuffers()
            .create_pipeline(config)?;

        Ok(pipeline)
    }

    // --- Render pass -------------------------------------------------------

    fn create_renderpass(&mut self) -> &mut Self {
        let attachments = [
            // Color attachment
            vk::AttachmentDescription {
                format: self.ctx.surface_format().format,
                samples: vk::SampleCountFlags::TYPE_1,
                load_op: vk::AttachmentLoadOp::CLEAR,
                store_op: vk::AttachmentStoreOp::STORE,
                stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
                stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
                initial_layout: vk::ImageLayout::UNDEFINED,
                final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
                ..Default::default()
            },
            // Depth attachment
            vk::AttachmentDescription {
                format: vk::Format::D16_UNORM,
                samples: vk::SampleCountFlags::TYPE_1,
                load_op: vk::AttachmentLoadOp::CLEAR,
                store_op: vk::AttachmentStoreOp::DONT_CARE,
                stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
                stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
                initial_layout: vk::ImageLayout::UNDEFINED,
                final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                ..Default::default()
            },
        ];

        let color_ref = [vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        }];
        let depth_ref = vk::AttachmentReference {
            attachment: 1,
            layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        };

        // Subpass dependency: wait for colour output from previous frame
        let dependencies = [vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL,
            dst_subpass: 0,
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            src_access_mask: vk::AccessFlags::empty(),
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE
                | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            dependency_flags: vk::DependencyFlags::empty(),
        }];

        let subpass = vk::SubpassDescription::default()
            .color_attachments(&color_ref)
            .depth_stencil_attachment(&depth_ref)
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS);

        self.renderpass = unsafe {
            self.ctx
                .device()
                .create_render_pass(
                    &vk::RenderPassCreateInfo::default()
                        .attachments(&attachments)
                        .subpasses(std::slice::from_ref(&subpass))
                        .dependencies(&dependencies),
                    None,
                )
                .expect("Failed to create render pass")
        };

        self
    }

    // --- Pipeline layout (push constants) ----------------------------------

    fn create_layout(&mut self) -> &mut Self {
        // One push constant range covering our full PushConstants struct
        let push_constant_range = vk::PushConstantRange {
            stage_flags: vk::ShaderStageFlags::VERTEX,
            offset: 0,
            size: std::mem::size_of::<PushConstants>() as u32,
        };

        self.layout = unsafe {
            self.ctx
                .device()
                .create_pipeline_layout(
                    &vk::PipelineLayoutCreateInfo::default()
                        .push_constant_ranges(std::slice::from_ref(&push_constant_range)),
                    None,
                )
                .expect("Failed to create pipeline layout")
        };
        self
    }

    // --- Framebuffers ------------------------------------------------------

    fn create_framebuffers(&mut self) -> &mut Self {
        let depth_view = self.ctx.depth_image_view();
        let resolution = self.ctx.surface_resolution();

        self.framebuffers = self
            .ctx
            .present_image_views()
            .iter()
            .enumerate()
            .map(|(i, &color_view)| {
                let attachments = [color_view, depth_view];
                unsafe {
                    self.ctx
                        .device()
                        .create_framebuffer(
                            &vk::FramebufferCreateInfo::default()
                                .render_pass(self.renderpass)
                                .attachments(&attachments)
                                .width(resolution.width)
                                .height(resolution.height)
                                .layers(1),
                            None,
                        )
                        .unwrap_or_else(|_| panic!("Failed to create framebuffer {}", i))
                }
            })
            .collect();
        self
    }

    // --- Graphics pipeline -------------------------------------------------

    fn create_pipeline(&mut self, cfg: &GraphicsPipelineConfig) -> anyhow::Result<&mut Self> {
        let vertex_shader =
            ShaderModule::load_from_file(Arc::clone(&self.ctx.device_ctx), &cfg.vert_shader_path)?;

        let fragment_shader = cfg
            .frag_shader_path
            .as_ref()
            .map(|p| ShaderModule::load_from_file(Arc::clone(&self.ctx.device_ctx), p))
            .transpose()?;

        let shader_entry = c"main";
        let mut stages = vec![vk::PipelineShaderStageCreateInfo {
            module: vertex_shader.module,
            p_name: shader_entry.as_ptr(),
            stage: vk::ShaderStageFlags::VERTEX,
            ..Default::default()
        }];
        if let Some(ref frag) = fragment_shader {
            stages.push(vk::PipelineShaderStageCreateInfo {
                module: frag.module,
                p_name: shader_entry.as_ptr(),
                stage: vk::ShaderStageFlags::FRAGMENT,
                ..Default::default()
            });
        }

        let attribute_descriptions = Vertex::get_attribute_descriptions();
        let binding_descriptions = [Vertex::get_binding_description()];
        let vertex_input = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_attribute_descriptions(&attribute_descriptions)
            .vertex_binding_descriptions(&binding_descriptions);

        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo {
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            ..Default::default()
        };

        let resolution = self.ctx.surface_resolution();
        self.viewports = vec![vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: resolution.width as f32,
            height: resolution.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];
        self.scissors = vec![resolution.into()];

        let viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .viewports(&self.viewports)
            .scissors(&self.scissors);

        let rasterization = vk::PipelineRasterizationStateCreateInfo {
            front_face: vk::FrontFace::COUNTER_CLOCKWISE,
            line_width: 1.0,
            polygon_mode: cfg.polygon_mode.unwrap_or(vk::PolygonMode::FILL),
            cull_mode: cfg.cull_mode.unwrap_or(vk::CullModeFlags::BACK),
            ..Default::default()
        };

        let multisample = vk::PipelineMultisampleStateCreateInfo {
            rasterization_samples: vk::SampleCountFlags::TYPE_1,
            ..Default::default()
        };

        let noop_stencil = vk::StencilOpState {
            fail_op: vk::StencilOp::KEEP,
            pass_op: vk::StencilOp::KEEP,
            depth_fail_op: vk::StencilOp::KEEP,
            compare_op: vk::CompareOp::ALWAYS,
            ..Default::default()
        };
        let depth_stencil = vk::PipelineDepthStencilStateCreateInfo {
            depth_test_enable: 1,
            depth_write_enable: 1,
            depth_compare_op: vk::CompareOp::LESS_OR_EQUAL,
            front: noop_stencil,
            back: noop_stencil,
            max_depth_bounds: 1.0,
            ..Default::default()
        };

        let color_blend_attachments = [vk::PipelineColorBlendAttachmentState {
            blend_enable: 0,
            color_write_mask: vk::ColorComponentFlags::RGBA,
            ..Default::default()
        }];
        let color_blend = vk::PipelineColorBlendStateCreateInfo::default()
            .attachments(&color_blend_attachments);

        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state =
            vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

        self.handle = unsafe {
            self.ctx
                .device()
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &[vk::GraphicsPipelineCreateInfo::default()
                        .stages(&stages)
                        .vertex_input_state(&vertex_input)
                        .input_assembly_state(&input_assembly)
                        .viewport_state(&viewport_state)
                        .rasterization_state(&rasterization)
                        .multisample_state(&multisample)
                        .depth_stencil_state(&depth_stencil)
                        .color_blend_state(&color_blend)
                        .dynamic_state(&dynamic_state)
                        .layout(self.layout)
                        .render_pass(self.renderpass)],
                    None,
                )
                .expect("Failed to create graphics pipeline")[0]
        };

        Ok(self)
    }
}

impl Drop for GraphicsPipeline {
    fn drop(&mut self) {
        unsafe {
            let device = self.ctx.device();
            for &fb in &self.framebuffers {
                device.destroy_framebuffer(fb, None);
            }
            device.destroy_render_pass(self.renderpass, None);
            device.destroy_pipeline_layout(self.layout, None);
            device.destroy_pipeline(self.handle, None);
        }
    }
}
