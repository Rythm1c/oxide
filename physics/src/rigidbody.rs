use math::{mat4x4::Mat4x4, quaternion::Quat, transform::Transform, vec3::Vec3};

pub struct RigidBody {
    inv_mass: f32,

    linear_damping: f32,

    position: Vec3,

    orientation: Quat,

    velocity: Vec3,

    rotation: Vec3,
    //transform: Transform,
}

impl Default for RigidBody {
    fn default() -> Self {
        Self {
            inv_mass: 1.0,
            linear_damping: 0.3,
            position: Vec3::ZERO,
            orientation: Quat::ZERO,
            velocity: Vec3::ZERO,
            rotation: Vec3::ZERO,
            //transform: Transform::default(),
        }
    }
}

impl RigidBody {
    pub fn inv_mass(mut self, value: f32) -> Self {
        self.inv_mass = value;
        self
    }

    pub fn linear_damping(mut self, value: f32) -> Self {
        self.linear_damping = value;
        self
    }

    pub fn position(mut self, value: Vec3) -> Self {
        self.position = value;
        self
    }

    pub fn orientation(mut self, value: Quat) -> Self {
        self.orientation = value;
        self
    }

    pub fn velocity(mut self, value: Vec3) -> Self {
        self.velocity = value;
        self
    }

    pub fn rotation(mut self, value: Vec3) -> Self {
        self.rotation = value;
        self
    }

    //TODO: Complete this function
    //pub fn calculate_derived_data(&mut self) {}

    // combines rotation and translation into a tranform matrix
    pub fn transform_matrix(&self) -> Mat4x4 {
        Transform::default()
            .translation(self.position)
            .orientation(self.orientation.normalize())
            .to_mat()
    }
}
