use math::{
    mat4x4::{self, Mat4x4},
    quaternion::Quat,
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
    orientation: Quat,
    pitch      : f32,
    yaw        : f32,

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
        Self {
            pos        : vec3(0.0, 3.0, -10.0),
            orientation: Quat::ZERO,
            pitch      : 0.0,
            yaw        : 0.0,

            aspect_ratio,

            fov        : 45.0,
            near       : 0.1,
            far        : 100.0,
            motion     : CameraMotion::Still,
            sensitivity: 0.075,
            speed      : 15.0,
        }
    }

    // ==================== Input Handling ====================
    pub fn rotate(&mut self, dx: f32, dy: f32) {
        self.yaw += -dx * self.sensitivity;
        self.pitch += dy * self.sensitivity;

        // Clamp pitch to avoid flipping
        self.pitch = self.pitch.clamp(-89.0, 89.0);

        // Update orientation quaternion from euler angles
        let yaw_quat     = Quat::rotation_y(self.yaw);
        let pitch_quat   = Quat::rotation_x(self.pitch);
        self.orientation = yaw_quat * pitch_quat;
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

    pub fn update(&mut self, delta: f32) {
        match self.motion {
            CameraMotion::Still     => {}
            CameraMotion::Forwards  => self.move_forward(delta),
            CameraMotion::BackWards => self.move_forward(-delta),
            CameraMotion::Left      => self.strafe(-delta),
            CameraMotion::Right     => self.strafe(delta),
            CameraMotion::Up        => self.move_up(delta),
            CameraMotion::Down      => self.move_up(-delta),
        }
    }

    // ==================== Movement ====================
    fn move_forward(&mut self, delta: f32) {
        self.pos = self.pos + self.get_forward() * self.speed * delta;
    }

    fn strafe(&mut self, delta: f32) {
        let right = cross(&self.get_forward(), &Vec3::Y);
        self.pos = self.pos + right * self.speed * delta;
    }

    fn move_up(&mut self, delta: f32) {
        self.pos.y += self.speed * delta;
    }

    // ==================== Getters ====================
    pub fn get_forward(&self) -> Vec3 {
        (self.orientation * Vec3::Z).unit()
    }

    pub fn position(&self) -> Vec3 {
        self.pos
    }

    pub fn view_matrix(&self) -> Mat4x4 {
        mat4x4::look_at(self.pos, self.pos + self.get_forward(), Vec3::Y)
    }

    pub fn projection_matrix(&self) -> Mat4x4 {
        let mut projection = mat4x4::perspective(self.fov, self.aspect_ratio, self.near, self.far);
        
        projection.data[1][1] *= -1.0; // Flip Y for Vulkan's coordinate system

        projection
    }

    pub fn view_projection_matrix(&self) -> Mat4x4 {
        mat4x4::transpose(&(self.projection_matrix() * self.view_matrix()))
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
        let dir = self.get_forward();

        CameraUbo {
            view_dir : [dir.x, dir.y, dir.z, 0.0],
            view     : mat4x4::transpose(&self.view_matrix()).data,
            proj     : mat4x4::transpose(&self.projection_matrix()).data,
        }
    }
}
