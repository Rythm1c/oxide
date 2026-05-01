pub mod cube;
pub mod cube_sphere;
pub mod torus;
pub mod triangle;
pub mod uv_sphere;

use engine_core::vertex::Vertex;

pub enum Shape {
    Cube {
        size : f32,
        color: Option<[f32; 3]>,
    },
    
    UVSphere {
        radius  : f32,
        segments: u32,
        rings   : u32,
        color   : Option<[f32; 3]>,
    },

    CubeSphere { 
        radius      : f32, 
        subdivisions: u32, 
        color       : Option<[f32; 3]> 
    }
    /* 

    Torus { major_radius: f32, minor_radius: f32, major_segments: u32, minor_segments: u32, color: Option<[f32; 3]> },
    , */
}

impl Shape {
    /// Generates vertex and index data for this shape.
    pub fn generate(&self) -> (Vec<Vertex>, Vec<u32>) {
        match self {
            Shape::Cube { size, color } => cube::generate_cube(*size, *color),

            Shape::UVSphere {
                radius,
                segments,
                rings,
                color,
            } => uv_sphere::generate_uv_sphere(*radius, *segments, *rings, *color), 

            Shape::CubeSphere { 
                radius, 
                subdivisions, 
                color } => {
                cube_sphere::generate_cube_sphere(*radius, *subdivisions, *color)
            }
        }
    }
}

#[derive(Clone)]
/// Represents mesh geometry with vertices and indices for GPU rendering.
pub struct Geometry {
    vertices: Vec<Vertex>,
    indices : Vec<u32>,
}

impl Geometry {
    /// Creates geometry from a shape definition.
    pub fn new(shape: Shape) -> Self {
        let (vertices, indices) = shape.generate();
        Self { vertices, indices }
    }

    /// Returns a slice of the vertices.
    pub fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }

    /// Returns a slice of the indices.
    pub fn indices(&self) -> &[u32] {
        &self.indices
    }

    /// Returns the number of vertices.
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Returns the number of indices.
    pub fn index_count(&self) -> usize {
        self.indices.len()
    }
}
