pub mod cube;
pub mod cube_sphere;
pub mod torus;
pub mod triangle;
pub mod uv_sphere;

use std::sync::Arc;

use engine_core::buffer::{Buffer, BufferUsage};
use engine_core::device::DeviceContext;
use engine_core::vertex::Vertex;

pub struct Geometry {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}
impl Geometry {
    pub fn new(geometry_type: GeometryType) -> Self {
        let (vertices, indices) = geometry_type.generate();
        Self { vertices, indices }
    }

    pub fn vertices(&self) -> &Vec<Vertex> {
        &self.vertices
    }

    pub fn indices(&self) -> &Vec<u32> {
        &self.indices
    }

    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    pub fn index_count(&self) -> usize {
        self.indices.len()
    }

    pub fn vertex_buffer(&self, ctx: Arc<DeviceContext>) -> anyhow::Result<Buffer> {
        Buffer::new_with_data(ctx, &self.vertices, BufferUsage::VERTEX)
    }

    pub fn index_buffer(&self, ctx: Arc<DeviceContext>) -> anyhow::Result<Buffer> {
        Buffer::new_with_data(ctx, &self.indices, BufferUsage::INDEX)
    }
}
pub enum GeometryType {
    Cube { size: f32, color: Option<[f32; 3]> },
}

impl GeometryType {
    pub fn generate(&self) -> (Vec<Vertex>, Vec<u32>) {
        match self {
            GeometryType::Cube { size, color } => cube::generate_cube(*size, *color),
        }
    }
}
