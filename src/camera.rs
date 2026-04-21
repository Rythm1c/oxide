use math::mat4::{Mat4, look_at, transpose};
use math::vec3::{Vec3, cross, vec3};

pub struct Camera {
    pub pos        : Vec3,
    pub up         : Vec3,
    pub speed      : f32,
    pub sensitivity: f32,
    pub direction  : Vec3,
}

impl Camera {
    pub fn new() -> Self {
        Camera {
            pos        : Vec3::new(0.0, 0.0, 3.0),
            up         : Vec3::new(0.0, 1.0, 0.0),
            speed      : 2.5,
            sensitivity: 0.1,
            direction  : Vec3::new(0.0, 0.0, 0.0),
        }
    }

    pub fn view_matrix(&self) -> Mat4 {
        let mut view = look_at(&self.pos, &(self.pos + self.direction), &self.up);
        view.data[1][1] *= -1.0; // Invert Y axis for Vulkan's coordinate system
        // vulkan expects column-major order, so we need to transpose the matrix before returning it
        transpose(&view)
    }

    pub fn rotate(&mut self, yaw: f32, pitch: f32) {
        let yaw = yaw * self.sensitivity;
        let pitch = pitch * self.sensitivity;

        let direction = vec3(
            yaw.cos() * pitch.cos(),
            pitch.sin(),
            yaw.sin() * pitch.cos(),
        );
        self.direction = direction.unit();
    }

    pub fn move_camera(&mut self, movement: CameraMovement) {
        match movement {
            CameraMovement::Forward  => self.pos = self.pos + self.direction * self.speed,
            CameraMovement::Backward => self.pos = self.pos - self.direction * self.speed,
            CameraMovement::Left     => {
                let left = cross(&self.direction, &self.up).unit();
                self.pos = self.pos - left * self.speed;
            }
            CameraMovement::Right => {
                let right = cross(&self.direction, &self.up).unit();
                self.pos = self.pos + right * self.speed;
            }
        }
    }
}

pub enum CameraMovement {
    Forward,
    Backward,
    Left,
    Right,
}
