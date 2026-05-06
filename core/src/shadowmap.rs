use std::sync::Arc;

use anyhow::Ok;
use ash::vk;

use super::pipeline::ShadowPipeline;
use crate::{device::DeviceContext, texture::Texture};

pub struct ShadowMap {
    ctx: Arc<DeviceContext>,
    shadow_pipeline: ShadowPipeline,
    map: Texture,
    sampler: vk::Sampler,
}

impl ShadowMap {
    pub fn new(ctx: Arc<DeviceContext>, width: u32, height: u32) -> anyhow::Result<Self> {
        let resolution = vk::Extent2D::default().width(width).height(height);
        let map = Texture::create_depth(Arc::clone(&ctx), resolution, vk::Format::D32_SFLOAT)?;
        let sampler = Self::create_sampler(Arc::clone(&ctx));
        let shadow_pipeline = ShadowPipeline::new(Arc::clone(&ctx), map.view, resolution)?;

        Ok(Self {
            ctx: Arc::clone(&ctx),
            shadow_pipeline,
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
}

impl Drop for ShadowMap {
    fn drop(&mut self) {
        unsafe {
            self.ctx.device.destroy_sampler(self.sampler, None);
        }
    }
}
