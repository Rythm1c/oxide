use math::vec3::Vec3;

use crate::{collision::test_sphere_shere_intersection, rigidbody::RigidBody};

pub struct PhyWorld {
    pub rigid_bodies: Vec<RigidBody>,

    gravity: Vec3,
}

impl Default for PhyWorld {
    fn default() -> Self {
        Self {
            rigid_bodies: Vec::new(),
            gravity: Vec3::new(0.0, -9.8, 0.0),
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

    pub fn update(&mut self, dt: f32) -> anyhow::Result<()> {
        self.integrate(dt);
        self.test_collisions()?;
        Ok(())
    }

    fn test_collisions(&mut self) -> anyhow::Result<()> {
        for i in 0..self.rigid_bodies.len() {
            for j in (i + 1)..self.rigid_bodies.len() {
                let bodies = &mut self.rigid_bodies;

                if bodies[i].mass.is_infinite() && bodies[j].mass.is_infinite() {
                    continue;
                }

                if test_sphere_shere_intersection(&bodies[i], &bodies[j])? {
                    bodies[i].velocity = Vec3::ZERO;
                    bodies[j].velocity = Vec3::ZERO;
                }
            }
        }

        Ok(())
    }

    fn integrate(&mut self, dt: f32) {
        for body in self.rigid_bodies.iter_mut() {
            if body.mass.is_infinite() {
                continue;
            }
            // I = dp , F = dp / dt => dp = F * dt => I = F * dt
            // F = mg
            let impulse_gravity = self.gravity * body.mass * dt;
            body.apply_impulse_linear(impulse_gravity);

            // also update position with velocity
            body.position = body.position + body.velocity * dt;
        }
    }
}
