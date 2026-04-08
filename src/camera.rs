use math::mat4::{Mat4, look_at};
use math::vec3::{Vec3, cross, vec3};

pub struct Camera {
    pub pos: Vec3,
    pub up: Vec3,
    pub speed: f32,
    pub direction: Vec3,
}

impl Camera {
    pub fn new() -> Self {
        Camera {
            pos: Vec3::new(0.0, 0.0, 3.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            speed: 2.5,
            direction: Vec3::new(0.0, 0.0, 0.0),
        }
    }

    pub fn view_matrix(&self) -> Mat4 {
        look_at(&self.pos, &(self.pos + self.direction), &self.up)
    }

    pub fn rotate(&mut self, yaw: f32, pitch: f32) {
        let direction = vec3(
            yaw.to_radians().cos() * pitch.to_radians().cos(),
            pitch.to_radians().sin(),
            yaw.to_radians().sin() * pitch.to_radians().cos(),
        );
        self.direction = direction.unit();
    }

    pub fn move_camera(&mut self, movement: CameraMovement) {
        match movement {
            CameraMovement::Forward => self.pos = self.pos + self.direction * self.speed,
            CameraMovement::Backward => self.pos = self.pos - self.direction * self.speed,
            CameraMovement::Left => {
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
