use engine_core::buffer::{Buffer, BufferUsage};
use engine_core::device::DeviceContext;
use engine_core::drawable::RenderObject;
use engine_core::pipeline::PushConstants;
use engine_core::descriptor::{MaterialDescriptorSet, MaterialAllocator};

use engine_core::ubo::MaterialUbo;
use geometry::{Geometry, Shape};

use math::mat4x4;
use math::transform::Transform;

use std::sync::Arc;

/// A renderable object combining geometry, transform, and GPU buffers.
#[derive(Clone)]
pub struct Object {
    geometry     : Geometry,
    transform    : Transform,
    material     : Material,
    // gpu stuff
    vertex_buffer: Option<Arc<Buffer>>,
    index_buffer : Option<Arc<Buffer>>,
    gpu_material : Option<MaterialDescriptorSet>

}

impl Object {
    /// Creates a new Object from a Shape without GPU buffers.
    /// Call `upload_to_gpu()` to transfer data to the GPU.
    pub fn new(shape: Shape, m: Material) -> anyhow::Result<Self> {
        Ok(
        Self {
            geometry     : Geometry::new(shape),
            transform    : Transform::default(),
            material     : m,
            vertex_buffer: None,
            index_buffer : None,
            gpu_material : None
        })
    }

    pub fn upload_geometry_to_gpu(&mut self, ctx: Arc<DeviceContext>) -> anyhow::Result<()> {
        self.vertex_buffer = Some(Arc::new(Buffer::new_with_data(
            ctx.clone(),
            self.geometry.vertices(),
            BufferUsage::VERTEX,
        )?));

        if !self.geometry.indices().is_empty() {
            self.index_buffer = Some(Arc::new(Buffer::new_with_data(
                ctx,
                self.geometry.indices(),
                BufferUsage::INDEX,
            )?));
        }

        Ok(())
    }

    pub fn upload_material_to_gpu(
        &mut self, 
        material_allocator: &Arc<MaterialAllocator>) 
        -> anyhow::Result<()> {
        self.gpu_material = Some(material_allocator.allocate(&self.material.get_ubo())?);
        Ok(())
    }

    /// Checks if this object has been uploaded to GPU.
    pub fn is_uploaded(&self) -> bool {
        self.vertex_buffer.is_some()
    }
    /// Returns a reference to this object's geometry.
    pub fn geometry(&self) -> &Geometry {
        &self.geometry
    }

    /// Returns an immutable reference to this object's transform.
    pub fn transform(&self) -> &Transform {
        &self.transform
    }

    /// Returns a mutable reference to this object's transform.
    pub fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }

    /// Constructs a RenderObject from the current state for rendering.
    /// Returns an error if the object hasn't been uploaded to GPU yet.
    pub fn get_render_object(&self) -> anyhow::Result<RenderObject> {
        let vertex_buffer = self
            .vertex_buffer
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Object not uploaded to GPU. Call upload_to_gpu() first"))?;

        let model = mat4x4::transpose(&self.transform.to_mat()).data;

        Ok(RenderObject {
            vertex_buffer : Arc::clone(vertex_buffer),
            index_buffer  : self.index_buffer.as_ref().map(Arc::clone),
            vertex_count  : self.geometry.vertex_count() as u32,
            index_count   : self.geometry.index_count() as u32,
            push_constants: PushConstants { model },
        })
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Material {
    pub metallic   : f32,
    pub roughness  : f32,
    pub ao         : f32,
    // checker board stuff
    pub use_checker: bool,  // 0.0=solid, 1.0=checker
    pub divisions  : f32,   // number of checher boxes per face
    pub factor     : f32    // darkness of the checker boxes(0.0 - 1.0)
}
impl Default for Material {
    fn default() -> Self {
        Material { 
            metallic   : 0.5, 
            roughness  : 0.5, 
            ao         : 0.05, 
            use_checker: false, 
            divisions  : 0.0, 
            factor     : 1.0 }
    }
}

impl Material {

    pub fn polished() -> Self{
        let mut material = Material::default();
        material.use_checker = checkered;
        material.divisions   = divisions;
        material.factor      = factor;

        material.roughness   = 0.3;
        material.metallic    = 0.0;

        material
    }

    pub fn stone(checkered: bool,divisions: f32, factor: f32)-> Self {
        let mut material = Material::default();
        material.use_checker = checkered;
        material.divisions   = divisions;
        material.factor      = factor;

        material.roughness   = 0.95;
        material.metallic    = 0.0;

        material
    }

    pub fn metal(checkered: bool,divisions: f32, factor: f32)->Self{
        let mut material = Material::default();
        material.use_checker = checkered;
        material.divisions   = divisions;
        material.factor      = factor;

        material.metallic    = 0.95;
        material.roughness   = 0.1;

        material
    }

    pub fn rubber(checkered: bool,divisions: f32, factor: f32)->Self{
        let mut material = Material::default();
        material.use_checker = checkered;
        material.divisions   = divisions;
        material.factor      = factor;

        material .roughness  = 0.85;
        material.metallic    = 0.0;

        material
    }

    pub fn get_ubo(&self) -> MaterialUbo {
        MaterialUbo {
            metallic   : self.metallic,
            roughness  : self.roughness,
            ao         : self.ao,

            _pad0      : 0.0,

            use_checker: if(self.use_checker) { 1.0 } else { 0.0 },
            divisions  : self.divisions,
            factor     : self.factor,

            _pad1      : 0.0
        }
    }
}
