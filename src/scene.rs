use super::camera::{Camera, CameraMovement};

use engine_core::drawable::RenderObject;

pub struct Scene {
    pub camera: Camera,
    pub objects: Vec<RenderObject>,
}

impl Scene {
    pub fn new() -> Self {
        Scene {
            camera: Camera::new(),
            objects: Vec::new(),
        }
    }

    pub fn add_object(&mut self, object: RenderObject) {
        self.objects.push(object);
    }

    pub fn objects(&self) -> &Vec<RenderObject> {
        &self.objects
    }

    pub fn objects_mut(&mut self) -> &mut Vec<RenderObject> {
        &mut self.objects
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    pub fn move_camera(&mut self, movement: CameraMovement) {
        self.camera.move_camera(movement);
    }

    pub fn update(&mut self, delta_time: f32) {
        // Placeholder for any per-frame scene updates (e.g. animations)
    }

    pub fn rotate_camera(&mut self, yaw: f32, pitch: f32) {
        self.camera.rotate(yaw, pitch);
    }

    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }
}
