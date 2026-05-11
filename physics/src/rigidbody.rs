use math::{mat3x3::Mat3x3, mat4x4::Mat4x4, quaternion::Quat, transform::Transform, vec3::Vec3};

use crate::collider::ColliderType;

pub struct RigidBody {
    pub mass: f32,

    pub resitution: f32,

    pub position: Vec3,

    pub orientation: Quat,

    /// linear velocity
    pub velocity: Vec3,

    pub rotation: Vec3,

    pub collider_type: Option<ColliderType>,
}

impl Default for RigidBody {
    fn default() -> Self {
        Self {
            mass: 10.0,
            resitution: 0.5,
            position: Vec3::ZERO,
            orientation: Quat::ZERO,
            velocity: Vec3::ZERO,
            rotation: Vec3::ZERO,
            collider_type: None,
        }
    }
}

impl RigidBody {
    pub fn get_center_of_mass_body_space(&self) -> anyhow::Result<Vec3> {
        self.collider_type
            .as_ref()
            .map(|ct| ct.get_center_of_mass())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "body collider type not set, cannot get center of mass in world space"
                )
            })
    }

    pub fn get_center_of_mass_world_space(&self) -> anyhow::Result<Vec3> {
        let cmbs = self.get_center_of_mass_body_space()?;
        Ok(self.position + self.orientation * cmbs)
    }

    pub fn get_inverse_inertia_tensor(&self) -> anyhow::Result<Mat3x3> {
        self.collider_type
            .as_ref()
            .map(|iit| iit.get_inverse_inertia_tensor())
            .ok_or_else(|| {
                anyhow::anyhow!("body collider type not set, cannot get inverse inertia tensor")
            })
    }

    pub fn get_inv_mass(&self) -> f32 {
        1.0 / self.mass
    }

    // combines rotation and translation into a tranform matrix
    pub fn get_transform_matrix(&self) -> Mat4x4 {
        Transform::default()
            .translation(self.position)
            .orientation(self.orientation.normalize())
            .to_mat()
    }

    pub fn apply_impulse_linear(&mut self, impulse: Vec3) {
        self.velocity = self.velocity + impulse * self.get_inv_mass();
    }

    pub fn mass(mut self, value: f32) -> Self {
        self.mass = value;
        self
    }

    pub fn resitution(mut self, value: f32) -> Self {
        self.resitution = value;
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

    pub fn collider_type(mut self, collider_type: ColliderType) -> Self {
        self.collider_type = Some(collider_type);
        self
    }
}
