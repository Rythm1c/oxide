/* use ash::vk;
use engine_core::vertex::Vertex;
use engine_core::{buffer::*, context::VkContext};

use math::vec3::Vec3;

pub struct Cube {
    pub position: Vec3,
    pub size: f32,
    pub color: [f32; 3],
    pub vertex_buffer: VertexBUffer,
    pub index_buffer: IndexBuffer,
}

impl Cube {
    pub fn new(context: &VkContext, position: Vec3, size: f32, color: [f32; 3]) -> Self {
        let half_size = size / 2.0;
        //vertices for with normals for each face(+X, -X, +Y, -Y, +Z, -Z)
        // Each face has its own 4 vertices (with normals pointing outwards), so 24 vertices total.
        // Indices define 2 triangles per face (6 faces, 12 triangles, 36 indices).

        let vertices = [
            // +X face
            Vertex::new([half_size, -half_size, -half_size], [1.0, 0.0, 0.0]),
            Vertex::new([half_size, half_size, -half_size], [1.0, 0.0, 0.0]),
            Vertex::new([half_size, half_size, half_size], [1.0, 0.0, 0.0]),
            Vertex::new([half_size, -half_size, half_size], [1.0, 0.0, 0.0]),
            // -X face
            Vertex::new([-half_size, -half_size, half_size], [-1.0, 0.0, 0.0]),
            Vertex::new([-half_size, half_size, half_size], [-1.0, 0.0, 0.0]),
            Vertex::new([-half_size, half_size, -half_size], [-1.0, 0.0, 0.0]),
            Vertex::new([-half_size, -half_size, -half_size], [-1.0, 0.0, 0.0]),
            // +Y face
            Vertex::new([-half_size, half_size, -half_size], [0.0, 1.0, 0.0]),
            Vertex::new([-half_size, half_size, half_size], [0.0, 1.0, 0.0]),
            Vertex::new([half_size, half_size, half_size], [0.0, 1.0, 0.0]),
            Vertex::new([half_size, half_size, -half_size], [0.0, 1.0, 0.0]),
            // -Y face
            Vertex::new([-half_size, -half_size, half_size], [0.0, -1.0, 0.0]),
            Vertex::new([-half_size, -half_size, -half_size], [0.0, -1.0, 0.0]),
            Vertex::new([half_size, -half_size, -half_size], [0.0, -1.0, 0.0]),
            Vertex::new([half_size, -half_size, half_size], [0.0, -1.0, 0.0]),
            // +Z face
            Vertex::new([half_size, -half_size, half_size], [0.0, 0.0, 1.0]),
            Vertex::new([half_size, half_size, half_size], [0.0, 0.0, 1.0]),
            Vertex::new([-half_size, half_size, half_size], [0.0, 0.0, 1.0]),
            Vertex::new([-half_size, -half_size, half_size], [0.0, 0.0, 1.0]),
            // -Z face
            Vertex::new([-half_size, -half_size, -half_size], [0.0, 0.0, -1.0]),
            Vertex::new([-half_size, half_size, -half_size], [0.0, 0.0, -1.0]),
            Vertex::new([half_size, half_size, -half_size], [0.0, 0.0, -1.0]),
            Vertex::new([half_size, -half_size, -half_size], [0.0, 0.0, -1.0]),
        ];

        let indices: [u32; 36] = [
            // +X face
            0, 1, 2, 0, 2, 3, // -X face
            4, 5, 6, 4, 6, 7, // +Y face
            8, 9, 10, 8, 10, 11, // -Y face
            12, 13, 14, 12, 14, 15, // +Z face
            16, 17, 18, 16, 18, 19, // -Z face
            20, 21, 22, 20, 22, 23,
        ];

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
        Cube {
            position,
            size,
            color,
            vertex_buffer,
            index_buffer,
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