use super::camera::Camera;

use engine_core::drawable::RenderObject;
use engine_core::ubo::{CameraUbo, LightUbo};

pub struct Scene {
    pub light: Light,
    pub camera: Camera,
    pub objects: Vec<RenderObject>,
}

impl Scene {
    pub fn new() -> Self {
        Scene {
            light: Light::default(),
            camera: Camera::new(800.0 / 600.0),
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

    pub fn handle_keyboard(&mut self, key: winit::keyboard::KeyCode, pressed: bool) {
        if !pressed {
            self.camera.set_motion_still();
            return; // Only handle key press, not release
        }

        match key {
            winit::keyboard::KeyCode::KeyW => {
                self.camera.set_motion_forwards();
            }
            winit::keyboard::KeyCode::KeyS => {
                self.camera.set_motion_backwards();
            }
            winit::keyboard::KeyCode::KeyA => {
                self.camera.set_motion_left();
            }
            winit::keyboard::KeyCode::KeyD => {
                self.camera.set_motion_right();
            }
            winit::keyboard::KeyCode::Space => {
                self.camera.set_motion_up();
            }
            winit::keyboard::KeyCode::ControlLeft | winit::keyboard::KeyCode::ControlRight => {
                self.camera.set_motion_down();
            }

            _ => {}
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        self.camera.update(delta_time);
        // Placeholder for any per-frame scene updates (e.g. animations)
    }

    pub fn rotate_camera(&mut self, yaw: f32, pitch: f32) {
        self.camera.rotate(yaw, pitch);
    }

    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }

    pub fn camera_ubo(&self) -> CameraUbo {
        self.camera.get_ubo()
    }

    pub fn light_ubo(&self) -> LightUbo {
        self.light.get_ubo()
    }
}

pub struct Light {
    pub color    : [f32; 3],
    pub direction: [f32; 3],
}

impl Default for Light {
    fn default() -> Self {
        Light {
            color    : [1.0, 1.0, 1.0],
            direction: [0.1, -1.0, -0.5],
        }
    }
}

impl Light {
    pub fn get_ubo(&self) -> LightUbo {
        let dir = self.direction;
        let col = self.color;
        LightUbo {
            light_dir  : [dir[0], dir[1], dir[2], 0.0],
            light_color: [col[0], col[1], col[2], 1.0],
        }
    }
}
