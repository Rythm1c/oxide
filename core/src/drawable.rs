use crate::buffer::Buffer;
use crate::pipeline::PushConstants;
use std::sync::Arc;

pub struct RenderObject {
    pub vertex_buffer: Arc<Buffer>,
    pub index_buffer: Option<Arc<Buffer>>,
    pub vertex_count: u32,
    pub index_count: u32,
    pub push_constants: PushConstants,
}
