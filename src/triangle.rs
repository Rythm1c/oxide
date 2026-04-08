/* use ash::vk;
use engine_core::vertex::Vertex;
use engine_core::{buffer::*, context::VkContext};

#[derive(Debug, Clone)]
pub struct Triangle {
    pub vertex_buffer: VertexBUffer,
    pub index_buffer: IndexBuffer,
}

impl Triangle {
    pub fn new(context: &VkContext) -> Self {
        let vertices = vec![
            Vertex {
                position: [-1.0, -1.0, 0.0],
                color: [1.0, 1.0, 0.0],
            }, // Bottom left
            Vertex {
                position: [1.0, -1.0, 0.0],
                color: [1.0, 0.0, 1.0],
            }, // Bottom right
            Vertex {
                position: [0.0, 1.0, 0.0],
                color: [0.0, 1.0, 1.0],
            }, // Top
        ];
        let indices = vec![0, 1, 2]; // Single triangle
        let vertex_buffer = VertexBUffer::create(
            context,
            vertices,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );
        let index_buffer = IndexBuffer::create(
            context,
            indices,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );
        Triangle {
            vertex_buffer,
            index_buffer, // Single triangle
        }
    }

    pub fn destroy(&mut self, context: &VkContext) {
        unsafe {
            self.vertex_buffer.destroy(context);
            self.index_buffer.destroy(context);
        }
    }
}
 */