use crate::object::Object;
use crate::camera::Camera;

use engine_core::descriptor::MaterialAllocator;
use engine_core::drawable::RenderObject;
use engine_core::ubo::{CameraUbo, LightUbo};
use engine_core::device::DeviceContext;

use math::quaternion::Quat;
use math::vec3::Vec3;

use std::sync::Mutex;
use std::sync::Arc;

pub struct Scene {
    pub light: Light,
    pub camera: Mutex<Camera>,
    pub objects: Mutex<Vec<Object>>,
}

impl Scene {
    /// Creates a new Scene with default camera and light.
    pub fn new() -> Self {
        Scene {
            light: Light::default(),
            camera: Mutex::new(Camera::new(800.0 / 600.0)),
            objects: Mutex::new(Vec::new()),
        }
    }

    /// Adds an object to the scene (thread-safe).
    pub fn add_object(&self, object: Object, pos: Vec3, scale: Vec3, rot: Quat) {
        let mut obj = object;
        obj.transform_mut().translation = pos;
        obj.transform_mut().scaling     = scale;
        obj.transform_mut().orientation = rot;
        self.objects.lock().unwrap().push(obj);
    }

    /// Returns render objects for all scene objects.
    pub fn render_objects(&self) -> Vec<RenderObject> {
        self.objects
            .lock()
            .unwrap()
            .iter()
            .filter_map(|obj| obj.get_render_object().ok())
            .collect()
    }

    /// Returns a reference to the scene's camera.
    pub fn camera(&self) -> Camera {
        *self.camera.lock().unwrap()
    }

    /// Handles keyboard input for camera control.
    pub fn handle_keyboard(&self, key: winit::keyboard::KeyCode, pressed: bool) {
        if !pressed {
            self.camera.lock().unwrap().set_motion_still();
            return; // Only handle key press, not release
        }

        let mut cam = self.camera.lock().unwrap();
        match key {
            winit::keyboard::KeyCode::KeyW => {
                cam.set_motion_forwards();
            }
            winit::keyboard::KeyCode::KeyS => {
                cam.set_motion_backwards();
            }
            winit::keyboard::KeyCode::KeyA => {
                cam.set_motion_left();
            }
            winit::keyboard::KeyCode::KeyD => {
                cam.set_motion_right();
            }
            winit::keyboard::KeyCode::Space => {
                cam.set_motion_up();
            }
            winit::keyboard::KeyCode::ControlLeft | 
            winit::keyboard::KeyCode::ControlRight => {
                cam.set_motion_down();
            }

            _ => {}
        }
    }

    /// Updates the scene for the current frame.
    pub fn update(&self, delta_time: f32) {
        self.camera.lock().unwrap().update(delta_time);
        // Placeholder for any per-frame scene updates (e.g. animations)
    }

    /// Rotates the camera based on mouse movement.
    pub fn rotate_camera(&self, yaw: f32, pitch: f32) {
        self.camera.lock().unwrap().rotate(yaw, pitch);
    }

    /// Returns camera UBO data.
    pub fn camera_ubo(&self) -> CameraUbo {
        self.camera.lock().unwrap().get_ubo()
    }

    /// Returns light UBO data.
    pub fn light_ubo(&self) -> LightUbo {
        let cpos = self.camera().position().to_array();
        let dir = self.light.direction;
        let col = self.light.color;
        let amb = self.light.ambient;
        LightUbo {
            camera_pos : [cpos[0], cpos[1], cpos[2], 0.0],
            ambient    : [amb[0], amb[1], amb[2], 0.0],
            light_dir  : [dir[0], dir[1], dir[2], 0.0],
            light_color: [col[0], col[1], col[2], 1.0],
        }
    }

    /// Uploads all scene objects to GPU in batch.
    /// Call this once after adding all objects, typically right before rendering.
    pub fn upload_all_objects(
        &self, 
        device_ctx: Arc<DeviceContext>, 
        material_allocator: &mut MaterialAllocator) 
        -> anyhow::Result<()> {
        let mut objects = self.objects.lock().unwrap();
        for obj in objects.iter_mut() {
            if !obj.is_uploaded() {
                obj.upload_geometry_to_gpu(Arc::clone(&device_ctx))?;
                obj.upload_material_to_gpu(material_allocator)?;
            }
        }
        Ok(())
    }
}

pub struct Light {
    pub color    : [f32; 3],
    pub ambient  : [f32; 3],
    pub direction: [f32; 3],
}

impl Default for Light {
    fn default() -> Self {
        Light {
            ambient  : [0.1; 3],
            color    : [10.0; 3],
            direction: [0.2, -1.0, -0.5],
        }
    }
}
