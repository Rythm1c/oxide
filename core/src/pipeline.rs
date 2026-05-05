use ash::vk::{self, DescriptorSetLayout};
use std::sync::Arc;

use super::context::VkContext;
use super::device::DeviceContext;
use crate::shader::ShaderModule;
use crate::vertex::Vertex;

// ---------------------------------------------------------------------------
// Push constants — 128 bytes guaranteed by the Vulkan spec.
// Holds model, view, and projection matrices (3 × 16 floats × 4 bytes = 192).
// Since that's over 128 bytes we pack MVP as a combined model-view-projection
// matrix (1 × 16 floats = 64 bytes) plus the model matrix for lighting (64).
//
// Layout (std430):
//   offset 64: mat4 model    (for normal/lighting transforms in the shader)
//
// In the vertex shader use:
//   layout(push_constant) uniform PushConstants {
//       mat4 model;
//   } pc;
// ---------------------------------------------------------------------------
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PushConstants {
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
        Self { model: id }
    }

    pub fn from_model_matrix(model: [[f32; 4]; 4]) -> Self {
        Self { model }
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self as *const Self as *const u8,
                std::mem::size_of::<Self>(),
            )
        }
    }

    pub fn push_range() -> vk::PushConstantRange {
        vk::PushConstantRange {
            stage_flags: vk::ShaderStageFlags::VERTEX,
            offset: 0,
            size: std::mem::size_of::<Self>() as u32,
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
    polygon_mode: Option<vk::PolygonMode>,
    cull_mode: Option<vk::CullModeFlags>,
    descriptor_layouts: Vec<DescriptorSetLayout>,
    push_ranges: Vec<vk::PushConstantRange>,
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

    pub fn descriptor_layouts(mut self, layouts: Vec<DescriptorSetLayout>) -> Self {
        self.descriptor_layouts = layouts;
        self
    }

    pub fn push_constant_ranges(mut self, ranges: Vec<vk::PushConstantRange>) -> Self {
        self.push_ranges = ranges;
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
            .create_layout(config)
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
                format: vk::Format::D32_SFLOAT,
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

    fn create_layout(&mut self, config: &GraphicsPipelineConfig) -> &mut Self {

        self.layout = unsafe {
            self.ctx
                .device()
                .create_pipeline_layout(
                    &vk::PipelineLayoutCreateInfo::default()
                        .set_layouts(&config.descriptor_layouts)
                        .push_constant_ranges(&config.push_ranges),
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
        let color_blend =
            vk::PipelineColorBlendStateCreateInfo::default().attachments(&color_blend_attachments);

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

pub struct ShadowPipeline {
    ctx: Arc<DeviceContext>,
    render_pass: vk::RenderPass,
    handle: vk::Pipeline,
    layout: vk::PipelineLayout,
    framebuffer: vk::Framebuffer,
}

impl ShadowPipeline {
    pub fn new(ctx: Arc<DeviceContext>, view: vk::ImageView, res: vk::Extent2D) -> Self {
        let render_pass = Self::create_renderpass(Arc::clone(&ctx));
        let framebuffer = Self::create_framebuffer(Arc::clone(&ctx), view, render_pass, res.clone());
        let layout = Self::create_pipeline_layout(Arc::clone(&ctx));
        let handle = Self::create_pipeline(Arc::clone(&ctx), render_pass);

        Self {
            ctx,
            render_pass,
            handle,
            layout,
            framebuffer,
        }
    }

    fn create_renderpass(ctx: Arc<DeviceContext>) -> vk::RenderPass {
        let depth_attachment = vk::AttachmentDescription::default()
            .format(vk::Format::D32_SFLOAT)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);

        let depth_ref = vk::AttachmentReference::default()
            .attachment(0)
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

        let subpass = vk::SubpassDescription::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&[])
            .depth_stencil_attachment(&depth_ref);

        let deps = [
            vk::SubpassDependency::default()
                .src_subpass(vk::SUBPASS_EXTERNAL)
                .dst_subpass(0)
                .src_stage_mask(vk::PipelineStageFlags::FRAGMENT_SHADER)
                .dst_stage_mask(vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
                .src_access_mask(vk::AccessFlags::SHADER_READ)
                .dst_access_mask(vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE)
                .dependency_flags(vk::DependencyFlags::BY_REGION),
            vk::SubpassDependency::default()
                .src_subpass(0)
                .dst_subpass(vk::SUBPASS_EXTERNAL)
                .src_stage_mask(vk::PipelineStageFlags::LATE_FRAGMENT_TESTS)
                .dst_stage_mask(vk::PipelineStageFlags::FRAGMENT_SHADER)
                .src_access_mask(vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE)
                .dst_access_mask(vk::AccessFlags::SHADER_READ)
                .dependency_flags(vk::DependencyFlags::BY_REGION),
        ];

        let attachments = [depth_attachment];
        let subpasses = [subpass];
        let create_info = vk::RenderPassCreateInfo::default()
            .attachments(&attachments)
            .subpasses(&subpasses)
            .dependencies(&deps);

        unsafe {
            ctx.device
                .create_render_pass(&create_info, None)
                .unwrap_or_else(|_| panic!("failed to create shadow map renderpass!"))
        }
    }
    fn create_framebuffer(
        ctx: Arc<DeviceContext>,
        depth_view: vk::ImageView,
        render_pass: vk::RenderPass,
        res: vk::Extent2D,
    ) -> vk::Framebuffer {
        let attachments = [depth_view];

        let create_info = vk::FramebufferCreateInfo::default()
            .attachment_count(1)
            .attachments(&attachments)
            .width(res.width)
            .height(res.height)
            .render_pass(render_pass);

        unsafe {
            ctx.device
                .create_framebuffer(&create_info, None)
                .unwrap_or_else(|_| panic!("Failed to create shadowmap framebuffer"))
        }
    }

    fn create_pipeline_layout(ctx: Arc<DeviceContext>) -> vk::PipelineLayout {
        let ranges = [PushConstants::push_range()];
        unsafe {
            ctx
                .device
                .create_pipeline_layout(
                    &vk::PipelineLayoutCreateInfo::default()
                        .push_constant_ranges(&ranges),// fix this
                    None,
                )
                .expect("Failed to create pipeline layout")
        }
    }

    fn create_pipeline(ctx: Arc<DeviceContext>, render_pass: vk::RenderPass) -> vk::Pipeline {
        let mut pipeline = vk::Pipeline::null();

        pipeline
    }
}

impl Drop for ShadowPipeline {
    fn drop(&mut self) {
        unsafe {
            self.ctx.device.destroy_render_pass(self.render_pass, None);
            self.ctx.device.destroy_framebuffer(self.framebuffer, None);
            self.ctx.device.destroy_pipeline_layout(self.layout, None);
            self.ctx.device.destroy_pipeline(self.handle, None);
        }
    }
}