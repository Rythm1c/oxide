use std::sync::Arc;

use anyhow::Ok;
use ash::vk;

use crate::{device::DeviceContext, texture::Texture};
use super::pipeline::ShadowPipeline;

pub struct ShadowMapConfig {
    ctx: Arc<DeviceContext>,
    vert_src: String,
    img_width: u32,
    img_height: u32,
}



pub struct ShadowMap {
    ctx: Arc<DeviceContext>,
    shadow_pipeline: ShadowPipeline,
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
        let map = Texture::create_depth(
            Arc::clone(&cfg.ctx),
            resolution.clone(),
            vk::Format::D32_SFLOAT,
        )?;
        let sampler = Self::create_sampler(Arc::clone(&cfg.ctx));

        let shadow_pipeline = ShadowPipeline::new(Arc::clone(&cfg.ctx), map.view, resolution);

        Ok(Self {
            ctx: Arc::clone(&cfg.ctx),
            shadow_pipeline,
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
}

impl Drop for ShadowMap {
    fn drop(&mut self) {
        unsafe {
            self.ctx.device.destroy_sampler(self.sampler, None);
        }
    }
}
