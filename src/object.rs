use engine_core::buffer::{Buffer, BufferUsage};
use engine_core::device::DeviceContext;
use engine_core::drawable::RenderObject;
use engine_core::pipeline::PushConstants;

use geometry::{Geometry, Shape};

use math::mat4x4;
use math::transform::Transform;

use std::sync::Arc;

/// A renderable object combining geometry, transform, and GPU buffers.
#[derive(Clone)]
pub struct Object {
    geometry: Geometry,
    transform: Transform,
    vertex_buffer: Option<Arc<Buffer>>,
    index_buffer: Option<Arc<Buffer>>,
}

impl Object {
    /// Creates a new Object from a Shape without GPU buffers.
    /// Call `upload_to_gpu()` to transfer data to the GPU.
    pub fn new(shape: Shape) -> Self {
        Self {
            geometry     : Geometry::new(shape),
            transform    : Transform::default(),
            vertex_buffer: None,
            index_buffer : None,
        }
    }

    /// Uploads this object's geometry to GPU buffers.
    pub fn upload_to_gpu(&mut self, ctx: Arc<DeviceContext>) -> anyhow::Result<()> {
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

