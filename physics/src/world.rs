use math::vec3::Vec3;

use crate::{
    collider::Collider,
    collision::{resolve_contact, test_sphere_sphere},
    rigidbody::RigidBody,
};

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
    pub fn add_body(&mut self, body: RigidBody) -> usize {
        let handle = self.rigid_bodies.len();
        self.rigid_bodies.push(body);
        handle // return handle so callers can reference the body later
    }

    pub fn change_gravity(&mut self, gravity: Vec3) {
        self.gravity = gravity;
    }

    pub fn update(&mut self, dt: f32) -> anyhow::Result<()> {
        self.integrate(dt);
        self.resolve_collisions()?;
        Ok(())
    }

    // ── Private ───────────────────────────────────────────────────────────────

    fn integrate(&mut self, dt: f32) {
        for body in self.rigid_bodies.iter_mut() {
            if body.mass.is_infinite() {
                continue;
            }
            // Gravity impulse:  Δp = F·Δt = m·g·Δt
            let gravity_impulse = self.gravity * body.mass * dt;
            body.apply_impulse_linear(gravity_impulse);
            body.update(dt);
        }
    }

    fn resolve_collisions(&mut self) -> anyhow::Result<()> {
        let n = self.rigid_bodies.len();
        for i in 0..n {
            for j in (i + 1)..n {
                // Skip pairs where both bodies have infinite mass (static geometry)
                if self.rigid_bodies[i].mass.is_infinite()
                    && self.rigid_bodies[j].mass.is_infinite()
                {
                    continue;
                }
                let contact = match (
                    &self.rigid_bodies[i].collider(),
                    &self.rigid_bodies[j].collider(),
                ) {
                    (Collider::Sphere(..), Collider::Sphere(..)) => {
                        test_sphere_sphere(i, j, &self.rigid_bodies)
                    }
                };

                resolve_contact(&contact, &mut self.rigid_bodies);
            }
        }
        Ok(())
    }
}
