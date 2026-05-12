use math::vec3::Vec3;

use crate::{collider::Collider, rigidbody::RigidBody};

type BodyHandle = usize;

// ── Contact data ──────────────────────────────────────────────────────────────

pub struct Contact {
    pub body_a: BodyHandle,
    pub body_b: BodyHandle,

    /// Contact point on A in world space
    pub pt_a_world: Vec3,
    /// Contact point on B in world space
    pub pt_b_world: Vec3,

    /// Collision normal pointing from B → A (i.e. push-A-away direction)
    pub normal: Vec3,

    pub has_collision: bool,
}

impl Contact {
    fn new(a: BodyHandle, b: BodyHandle) -> Self {
        Self {
            body_a: a,
            body_b: b,
            pt_a_world: Vec3::ZERO,
            pt_b_world: Vec3::ZERO,
            normal: Vec3::ZERO,
            has_collision: false,
        }
    }
}

// ── Narrow-phase detection ────────────────────────────────────────────────────

pub fn test_sphere_sphere(
    a: BodyHandle,
    b: BodyHandle,
    bodies: &[RigidBody],
) -> Contact {
    let mut contact = Contact::new(a, b);

    let ra = sphere_radius(bodies[a].collider()).unwrap();
    let rb = sphere_radius(bodies[b].collider()).unwrap();
    let rab = ra + rb;

    // AB = position_B - position_A
    let ab = bodies[b].position - bodies[a].position;
    if ab.len_sqrd() > rab * rab {
        return contact; // no collision
    }

    contact.has_collision = true;
    contact.normal = ab.normalize();
    contact.pt_a_world = bodies[a].position + contact.normal * ra;
    contact.pt_b_world = bodies[b].position - contact.normal * rb;

    contact
}

// ── Contact resolution ────────────────────────────────────────────────────────

pub fn resolve_contact(contact: &Contact, bodies: &mut Vec<RigidBody>) {
    if !contact.has_collision {
        return;
    }

    let a = contact.body_a;
    let b = contact.body_b;

    let inv_mass_a = bodies[a].inv_mass();
    let inv_mass_b = bodies[b].inv_mass();
    let total_inv_mass = inv_mass_a + inv_mass_b;

    let inv_inertia_a = bodies[a].inv_inertia_tensor_world();
    let inv_inertia_b = bodies[b].inv_inertia_tensor_body();

    let n = contact.normal;
    let p_on_a = contact.pt_a_world;
    let p_on_b = contact.pt_b_world;

    // Vectors from each centre-of-mass to the contact point
    let ra = p_on_a - bodies[a].center_of_mass_world();
    let rb = p_on_b - bodies[b].center_of_mass_world();

    // ── Restitution impulse ───────────────────────────────────────────────────
    let elasticity = bodies[a].restitution * bodies[b].restitution;

    // Effective velocity at the contact point (linear + angular contribution)
    let vel_a = bodies[a].velocity + bodies[a].angular_velocity.cross(&ra);
    let vel_b = bodies[b].velocity + bodies[b].angular_velocity.cross(&rb);
    let vab = vel_a - vel_b;

    // Angular contribution to the effective inverse mass at the contact point
    let ang_factor_a = (inv_inertia_a * ra.cross(&n)).cross(&ra);
    let ang_factor_b = (inv_inertia_b * rb.cross(&n)).cross(&rb);
    let angular_factor = (ang_factor_a + ang_factor_b).dot(&n);

    let j = -(1.0 + elasticity) * vab.dot(&n) / (total_inv_mass + angular_factor);
    let impulse = n * j;

    bodies[a].apply_impulse_at_point(impulse, p_on_a);
    bodies[b].apply_impulse_at_point(-impulse, p_on_b);

    // ── Friction impulse ─────────────────────────────────────────────────────
    let friction = bodies[a].friction * bodies[b].friction;

    // Recompute relative velocity after restitution impulse
    let vel_a = bodies[a].velocity + bodies[a].angular_velocity.cross(&ra);
    let vel_b = bodies[b].velocity + bodies[b].angular_velocity.cross(&rb);
    let vab = vel_a - vel_b;

    // Tangential (friction) direction
    let vel_normal = n * n.dot(&vab);
    let vel_tangent = vab - vel_normal;

    let tang_len_sq = vel_tangent.len_sqrd();
    if tang_len_sq < 1e-10 {
        // No tangential slip — skip friction
    } else {
        let tang_dir = vel_tangent.normalize();

        let ang_fric_a = (inv_inertia_a * ra.cross(&tang_dir)).cross(&ra);
        let ang_fric_b = (inv_inertia_b * rb.cross(&tang_dir)).cross(&rb);
        let inv_inertia_tang = (ang_fric_a + ang_fric_b).dot(&tang_dir);

        let reduced_mass = 1.0 / (total_inv_mass + inv_inertia_tang);
        let friction_impulse = vel_tangent * (reduced_mass * friction);

        bodies[a].apply_impulse_at_point(friction_impulse, p_on_a);
        bodies[b].apply_impulse_at_point(-friction_impulse, p_on_b);
    }

    // ── Positional correction (push overlapping bodies apart) ─────────────────
    let ds = contact.pt_b_world - contact.pt_a_world;
    let ta = inv_mass_a / total_inv_mass;
    let tb = inv_mass_b / total_inv_mass;

    bodies[a].position = bodies[a].position + ds * ta;
    bodies[b].position = bodies[b].position - ds * tb;
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn sphere_radius(ct: &Collider) -> anyhow::Result<f32> {
    match ct {
        Collider::Sphere(sphere) => Ok(sphere.radius),
        //_ => Err(anyhow::anyhow!("expected Sphere collider")),
    }
}
