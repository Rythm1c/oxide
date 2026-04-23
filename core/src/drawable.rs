use crate::buffer::Buffer;
use crate::pipeline::PushConstants;

pub struct RenderObject {
    pub vertex_buffer: Buffer,
    pub index_buffer: Option<Buffer>,
    pub vertex_count: u32,
    pub index_count: u32,
    pub push_constants: PushConstants,
}
