use ash::vk;

pub struct RenderObject {
    pub vertex_buffer: vk::Buffer,
    pub index_buffer: Option<vk::Buffer>,
    pub vertex_count: u32,
    pub index_count: u32,
}