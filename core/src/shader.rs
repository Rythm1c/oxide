use crate::device::DeviceContext;
use ash::vk;
use std::sync::Arc;
use ash::util::read_spv;

pub struct ShaderModule {
    pub module: vk::ShaderModule,
    pub ctx: Arc<DeviceContext>, // keep a ref to the device for cleanup
}

impl ShaderModule {
    pub fn new(ctx: Arc<DeviceContext>, code: &[u32]) -> anyhow::Result<Self> {
        let create_info = vk::ShaderModuleCreateInfo::default().code(code);
        let module = unsafe { ctx.device.create_shader_module(&create_info, None)? };
        Ok(Self { module, ctx })
    }

    pub fn load_from_file(ctx: Arc<DeviceContext>, path: &str) -> anyhow::Result<Self> {
        let code = read_shader_spv(path)?;
        Self::new(ctx, &code)
    }
}

impl Drop for ShaderModule {
    fn drop(&mut self) {
        unsafe {
            self.ctx.device.destroy_shader_module(self.module, None);
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