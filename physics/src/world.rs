use math::vec3::Vec3;

use crate::rigidbody::RigidBody;

pub struct PhyWorld {
    pub rigid_bodies: Vec<RigidBody>,

    gravity: Vec3,
}

impl Default for PhyWorld {
    fn default() -> Self {
        Self {
            rigid_bodies: Vec::new(),
            gravity: Vec3::new(0.0, -9.81, 0.0),
        }
    }
}

impl PhyWorld {
    pub fn add_body(&mut self, body: RigidBody) {
        self.rigid_bodies.push(body);
    }

    pub fn change_gravity(&mut self, gravity: Vec3) {
        self.gravity = gravity;
    }

    pub fn update(&mut self, dt: f32) {
        for body in self.rigid_bodies.iter_mut() {
            if body.inv_mass == 0.0 {
                continue;
            }
            // acceleration due to gravity
            body.velocity = body.velocity + self.gravity * dt;

            // also update position with velocity
            body.position = body.position + body.velocity * dt;
        }
    }
}
