use std::sync::Arc;

use anyhow::Ok;
use ash::vk;

use super::pipeline::ShadowPipeline;
use crate::{
    device::DeviceContext,
    drawable::{RenderObject, render_drawable},
    pipeline::ShadowMapPushConstants,
    texture::Texture,
};

pub struct ShadowMap {
    ctx: Arc<DeviceContext>,
    shadow_pipeline: ShadowPipeline,
    map: Texture,
    sampler: vk::Sampler,
    resolution: vk::Extent2D,
}

impl ShadowMap {
    pub fn new(ctx: Arc<DeviceContext>, width: u32, height: u32) -> anyhow::Result<Self> {
        let resolution = vk::Extent2D::default().width(width).height(height);
        let map = Texture::create_shadow_map(Arc::clone(&ctx), resolution, vk::Format::D32_SFLOAT)?;
        let sampler = Self::create_sampler(Arc::clone(&ctx));
        let shadow_pipeline = ShadowPipeline::new(Arc::clone(&ctx), map.view, resolution)?;

        Ok(Self {
            ctx: Arc::clone(&ctx),
            shadow_pipeline,
            map,
            sampler,
            resolution,
        })
    }

    fn create_sampler(ctx: Arc<DeviceContext>) -> vk::Sampler {
        let create_info = vk::SamplerCreateInfo::default()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_BORDER)
            .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_BORDER)
            .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_BORDER)
            .border_color(vk::BorderColor::FLOAT_OPAQUE_WHITE)
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

    pub fn pipeline(&self) -> vk::Pipeline {
        self.shadow_pipeline.handle()
    }

    pub fn pipeline_layout(&self) -> vk::PipelineLayout {
        self.shadow_pipeline.layout()
    }

    pub fn render_pass(&self) -> vk::RenderPass {
        self.shadow_pipeline.render_pass()
    }

    pub fn framebuffer(&self) -> vk::Framebuffer {
        self.shadow_pipeline.framebuffer()
    }

    pub fn sampler(&self) -> vk::Sampler {
        self.sampler
    }

    pub fn view(&self) -> vk::ImageView {
        self.map.view
    }

    pub fn begin_render_pass(&self, cmd: vk::CommandBuffer) {
        self.shadow_pipeline.begin_render_pass(cmd, self.resolution);
    }

    pub fn render_shadow(
        &self,
        cmd: vk::CommandBuffer,
        drawable: &RenderObject,
        light_space_matrix: [[f32; 4]; 4],
    ) {
        let push_constants = ShadowMapPushConstants::new(drawable.model_matrix, light_space_matrix);

        unsafe {
            self.ctx.device.cmd_push_constants(
                cmd,
                self.pipeline_layout(),
                vk::ShaderStageFlags::VERTEX,
                0,
                push_constants.as_bytes(),
            );

            render_drawable(&self.ctx.device, cmd, drawable);
        }
    }

    pub fn end_renderpass(&self, cmd: vk::CommandBuffer) {
        unsafe { self.ctx.device.cmd_end_render_pass(cmd) };
    }
}

impl Drop for ShadowMap {
    fn drop(&mut self) {
        unsafe {
            self.ctx.device.destroy_sampler(self.sampler, None);
        }
    }
}
