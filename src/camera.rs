use math::{
    mat4x4,
    vec3::{Vec3, cross, vec3},
};

use engine_core::ubo::CameraUbo;

#[derive(Clone, Copy, PartialEq)]
enum CameraMotion {
    Forwards,
    BackWards,
    Left,
    Right,
    Still,
    Up,
    Down,
}

/// Encapsulates all camera state and behavior
#[derive(Clone, Copy)]
pub struct Camera {
    // Position and orientation
    pos        : Vec3,
    front      : Vec3,
    right      : Vec3,
    pitch      : f32,
    yaw        : f32,
    up         : Vec3,
    //world_up   : Vec3,

    // Projection
    aspect_ratio: f32,
    fov         : f32,
    near        : f32,
    far         : f32,

    // Input
    motion     : CameraMotion,
    sensitivity: f32,
    speed      : f32,

}

impl Camera {
    pub fn new(aspect_ratio: f32) -> Self {
        let mut camera = Self {
            pos        : vec3(0.0, 1.0, 9.0),
            front      : -Vec3::Z,
            right      :  Vec3::X,
            pitch      : 0.0,
            yaw        : -90.0,
            up         : Vec3::Y,
            //world_up   : -Vec3::Y,

            aspect_ratio,

            fov        : 45.0,
            near       : 0.1,
            far        : 100.0,
            motion     : CameraMotion::Still,
            sensitivity: 0.075,
            speed      : 12.0,
        };
        camera.update_vectors();
        camera
    }

    // ==================== Input Handling ====================
    pub fn process_mouse(&mut self, dx: f32, dy: f32) {
        self.yaw   += dx * self.sensitivity;
        self.pitch -= dy * self.sensitivity;

        self.pitch = f32::clamp(self.pitch, -89.0, 89.0);

        self.update_vectors();
    }

    pub fn set_motion_still(&mut self) {
        self.motion = CameraMotion::Still;
    }
    pub fn set_motion_forwards(&mut self) {
        self.motion = CameraMotion::Forwards;
    }
    pub fn set_motion_backwards(&mut self) {
        self.motion = CameraMotion::BackWards;
    }
    pub fn set_motion_left(&mut self) {
        self.motion = CameraMotion::Left;
    }
    pub fn set_motion_right(&mut self) {
        self.motion = CameraMotion::Right;
    }
    pub fn set_motion_up(&mut self) {
        self.motion = CameraMotion::Up;
    }
    pub fn set_motion_down(&mut self) {
        self.motion = CameraMotion::Down;
    }

    pub fn process_keyboard(&mut self, delta: f32) {
        match self.motion {
            CameraMotion::Still     => {}
            CameraMotion::Forwards  => self.move_forward(delta),
            CameraMotion::BackWards => self.move_forward(-delta),
            CameraMotion::Left      => self.strafe(-delta),
            CameraMotion::Right     => self.strafe(delta),
            CameraMotion::Up        => self.move_up(delta),
            CameraMotion::Down      => self.move_up(-delta),
        }

        if self.motion != CameraMotion::Still{
            self.update_vectors();
        }
    }

    fn update_vectors(&mut self) {

        let yaw   = self.yaw.to_radians();
        let pitch = self.pitch.to_radians();

        self.front.x = yaw.cos() * pitch.cos();
        self.front.y = pitch.sin();
        self.front.z = yaw.sin() * pitch.cos();

        self.front = self.front.unit();

        self.right = cross(self.front, Vec3::Y).unit();

        self.up = cross(self.right, self.front).unit();

    }

    // ==================== Movement ====================
    fn move_forward(&mut self, delta: f32) {
        self.pos = self.pos + self.front * self.speed * delta;
    }

    fn strafe(&mut self, delta: f32) {
        self.pos = self.pos + self.right * self.speed * delta;
    }

    fn move_up(&mut self, delta: f32) {
        self.pos = self.pos + Vec3::Y * self.speed * delta;
    }

    // ==================== Getters ====================
    pub fn position(&self) -> Vec3 {
        self.pos
    }

    // ==================== Configuration ====================
    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio;
    }

    pub fn set_sensitivity(&mut self, sensitivity: f32) {
        self.sensitivity = sensitivity;
    }

    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed;
    }

    pub fn get_ubo(&self) -> CameraUbo {
        let mut view = mat4x4::look_at(self.pos, self.pos + self.front, self.up);
        view = mat4x4::transpose(&view); // Transpose for column-major order

        let mut proj = mat4x4::perspective(self.fov, self.aspect_ratio, self.near, self.far);
        proj.data[1][1] *= -1.0; // Flip Y for Vulkan's coordinate system
        proj = mat4x4::transpose(&proj); // Transpose for column-major order


        CameraUbo {
            view : view.data,
            proj : proj.data,
        }
    }
}
