use crate::buffer::Buffer;
use crate::descriptor::MaterialDescriptorSet;

use std::sync::Arc;

pub struct RenderObject {
    pub vertex_buffer : Arc<Buffer>,
    pub index_buffer  : Option<Arc<Buffer>>,
    pub vertex_count  : u32,
    pub index_count   : u32,
    pub model_matrix  : [[f32; 4]; 4],
    pub material_desc : Arc<MaterialDescriptorSet>
}
