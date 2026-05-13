use crate::camera::Camera;
use crate::object::{Material, Object};

use engine_core::descriptor::MaterialAllocator;
use engine_core::device::DeviceContext;
use engine_core::drawable::RenderObject;
use engine_core::ubo::{CameraUbo, LightUbo};

use geometry::Shape;
use math::mat4x4::Mat4x4;
use math::vec3::Vec3;
use physics::rigidbody::RigidBody;
use physics::world::PhyWorld;

use std::sync::Arc;
use std::sync::Mutex;

pub struct Scene {
    pub light: Light,
    pub camera: Mutex<Camera>,
    pub objects: Mutex<Vec<Object>>,
    physics_world: Mutex<PhyWorld>,
}

impl Scene {
    /// Creates a new Scene with default camera and light.
    pub fn new() -> Self {
        Scene {
            light: Light::default(),
            camera: Mutex::new(Camera::new(800.0 / 600.0)),
            objects: Mutex::new(Vec::new()),
            physics_world: Mutex::new(PhyWorld::default()),
        }
    }

    /// Adds an object to the scene (thread-safe).
    pub fn add_object(&self, material: Material, shape: Shape, body: RigidBody) {
        let mut object = Object::new(shape, material);
        object.transform_mut().translation = body.position;
        object.transform_mut().orientation = body.orientation;
        self.objects.lock().unwrap().push(object);

        self.physics_world.lock().unwrap().add_body(body);
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
            winit::keyboard::KeyCode::ControlLeft | winit::keyboard::KeyCode::ControlRight => {
                cam.set_motion_down();
            }

            _ => {}
        }
    }

    /// Updates the scene for the current frame.
    pub fn update(&self, delta_time: f32) {
        self.camera.lock().unwrap().process_keyboard(delta_time);
        // Placeholder for any per-frame scene updates (e.g. animations)
        self.physics_world.lock().unwrap().update(delta_time);
        self.sync_objects();
    }

    pub fn sync_objects(&self) {
        let objects = &mut self.objects.lock().unwrap();
        let obj_bodies = &self.physics_world.lock().unwrap().rigid_bodies;

        for i in 0..objects.len() {
            let obj_body = &obj_bodies[i];
            let object = &mut objects[i];

            object.transform_mut().translation = obj_body.position;
            object.transform_mut().orientation = obj_body.orientation;
        }
    }

    /// Rotates the camera based on mouse movement.
    pub fn rotate_camera(&self, yaw: f32, pitch: f32) {
        self.camera.lock().unwrap().process_mouse(yaw, pitch);
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
        let matrix = self.light.proj_view_matrix();
        LightUbo {
            camera_pos: [cpos[0], cpos[1], cpos[2], 0.0],
            light_dir: [dir[0], dir[1], dir[2], 0.0],
            light_color: [col[0], col[1], col[2], 1.0],
            light_space: matrix,
        }
    }

    /// Uploads all scene objects to GPU in batch.
    /// Call this once after adding all objects, typically right before rendering.
    pub fn upload_all_objects(
        &self,
        device_ctx: Arc<DeviceContext>,
        material_allocator: &mut MaterialAllocator,
    ) -> anyhow::Result<()> {
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
    pub color: [f32; 3],
    pub direction: [f32; 3],
}

impl Default for Light {
    fn default() -> Self {
        Light {
            color: [10.0; 3],
            direction: [0.6, -0.7, -0.5],
        }
    }
}

impl Light {
    pub fn proj_view_matrix(&self) -> [[f32; 4]; 4] {
        let direction = Vec3::from(&self.direction);
        let light_pos = Vec3::ZERO - direction.normalize() * 15.0;

        let view = Mat4x4::look_at(light_pos, Vec3::ZERO, Vec3::Y);

        let mut proj = Mat4x4::orthogonal(30.0, -30.0, 30.0, -30.0, -30.0, 30.0);
        proj.data[1][1] *= -1.0;

        let proj_view = proj * view;

        proj_view.transpose().data
    }
}
