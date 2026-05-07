use ash::vk;

use crate::buffer::Buffer;
use crate::descriptor::MaterialDescriptorSet;

use std::sync::Arc;

pub struct RenderObject {
    pub vertex_buffer: Arc<Buffer>,
    pub index_buffer: Option<Arc<Buffer>>,
    pub vertex_count: u32,
    pub index_count: u32,
    pub model_matrix: [[f32; 4]; 4],
    pub material_desc: Arc<MaterialDescriptorSet>,
}

pub fn render_drawable(device: &ash::Device, cmd: vk::CommandBuffer, drawable: &RenderObject) {
    unsafe {
        device.cmd_bind_vertex_buffers(cmd, 0, &[drawable.vertex_buffer.raw], &[0]);

        if let Some(idxbuf) = &drawable.index_buffer {
            device.cmd_bind_index_buffer(cmd, idxbuf.raw, 0, vk::IndexType::UINT32);
            device.cmd_draw_indexed(cmd, drawable.index_count, 1, 0, 0, 0);
        } else {
            device.cmd_draw(cmd, drawable.vertex_count, 1, 0, 0);
        }
    }
}
