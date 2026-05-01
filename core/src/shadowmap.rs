use std::sync::Arc;

use anyhow::Ok;
use ash::vk;

use crate::{device::DeviceContext, texture::Texture};

pub struct ShadowMapConfig {
    ctx: Arc<DeviceContext>,
    vert_src: String,
    img_width: u32,
    img_height: u32,
}

pub struct ShadowMap {
    ctx: Arc<DeviceContext>,
    render_pass: vk::RenderPass,
    pipeline: vk::Pipeline,
    framebuffer: vk::Framebuffer,
    resolution: vk::Extent2D,
    map: Texture,
    sampler: vk::Sampler,
}

impl ShadowMap {
    pub fn new(cfg: &ShadowMapConfig) -> anyhow::Result<Self> {
        let resolution = vk::Extent2D {
            width: cfg.img_width,
            height: cfg.img_height,
        };
        let map = Texture::create_depth(Arc::clone(&cfg.ctx), resolution.clone(), vk::Format::D32_SFLOAT)?;
        let sampler = Self::create_sampler(Arc::clone(&cfg.ctx));
        let render_pass = Self::create_renderpass(Arc::clone(&cfg.ctx));
        let framebuffer = Self::create_framebuffer(
            Arc::clone(&cfg.ctx),
            map.view,
            render_pass,
            resolution.clone(),
        );
        let pipeline = Self::create_pipeline(Arc::clone(&cfg.ctx), render_pass);

        Ok(Self {
            ctx: Arc::clone(&cfg.ctx),
            render_pass,
            pipeline,
            framebuffer,
            resolution,
            map,
            sampler,
        })
    }

    fn create_sampler(ctx: Arc<DeviceContext>) -> vk::Sampler {
        let create_info = vk::SamplerCreateInfo::default()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_BORDER)
            .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_BORDER)
            .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_BORDER)
            .compare_enable(true)
            .compare_op(vk::CompareOp::LESS_OR_EQUAL)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
            .mip_lod_bias(0.0)
            .min_lod(0.0)
            .max_lod(1.0)
            .anisotropy_enable(false);

        unsafe {
            ctx.device
                .create_sampler(&create_info, None)
                .unwrap_or_else(|_| panic!("failed to create shadow map sampler!"))
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
    fn create_pipeline(ctx: Arc<DeviceContext>, render_pass: vk::RenderPass) -> vk::Pipeline {
        let mut pipeline = vk::Pipeline::null();

        pipeline
    }
}

impl Drop for ShadowMap {
    fn drop(&mut self) {
        unsafe {
            self.ctx.device.destroy_render_pass(self.render_pass, None);
            self.ctx.device.destroy_framebuffer(self.framebuffer, None);
            self.ctx.device.destroy_pipeline(self.pipeline, None);
        }
    }
}
