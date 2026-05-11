use math::vec3::Vec3;

use crate::{collider::ColliderType, rigidbody::RigidBody};

type BodyHandle = usize;

pub struct Contact {
    pub pt_a_world_space: Vec3,
    pub pt_b_world_space: Vec3,

    pub pt_a_loacl_space: Vec3,
    pub pt_b_local_space: Vec3,

    pub collision: bool,

    pub normal: Vec3,

    pub body_a: BodyHandle,
    pub body_b: BodyHandle,
}

impl Contact {
    pub fn new(a: BodyHandle, b: BodyHandle) -> Self {
        Self {
            body_a: a,
            body_b: b,

            pt_a_world_space: Vec3::ZERO,
            pt_b_world_space: Vec3::ZERO,

            pt_a_loacl_space: Vec3::ZERO,
            pt_b_local_space: Vec3::ZERO,

            collision: false,

            normal: Vec3::ZERO,
        }
    }
}

pub fn test_sphere_shere_intersection(
    a: BodyHandle,
    b: BodyHandle,
    bodies: &Vec<RigidBody>,
) -> anyhow::Result<Contact> {
    let mut contact = Contact::new(a, b);
    // AB = AO + OB = OB - OA
    let ab = bodies[b].position - bodies[a].position;

    let ra = get_radius(bodies[a].collider_type.as_ref().unwrap())?;
    let rb = get_radius(bodies[b].collider_type.as_ref().unwrap())?;
    let rab = ra + rb;

    let ab_sqrd = ab.len_sqrd();
    if ab_sqrd <= (rab * rab) {
        contact.collision = true;
        contact.normal = ab.normalize();

        contact.pt_a_world_space = bodies[a].position + contact.normal * ra;
        contact.pt_b_world_space = bodies[b].position - contact.normal * rb;

        return Ok(contact);
    }

    contact.collision = false;
    Ok(contact)
}

pub fn resolve_contact(contact: &Contact, bodies: &mut Vec<RigidBody>) {
    if !contact.collision {
        return;
    }
    let a = contact.body_a;
    let b = contact.body_b;

    let inv_mass_a = bodies[a].get_inv_mass();
    let inv_mass_b = bodies[b].get_inv_mass();
    let total_inv_mass = inv_mass_a + inv_mass_b;

    let elastisity_a = bodies[a].resitution;
    let elasticity_b = bodies[b].resitution;
    let elasticity = elastisity_a * elasticity_b;

    // collision impulse
    let n = &contact.normal;
    let vab = bodies[a].velocity - bodies[b].velocity;
    let impulse_j = -(1.0 + elasticity) * vab.dot(n) / (total_inv_mass);
    let vec_impulse_j = *n * impulse_j;

    bodies[a].apply_impulse_linear(vec_impulse_j * 1.0);
    bodies[b].apply_impulse_linear(vec_impulse_j * -1.0);

    // move objects out of each other
    let ta = inv_mass_a / (total_inv_mass);
    let tb = inv_mass_b / (total_inv_mass);

    let ds = contact.pt_b_world_space - contact.pt_a_world_space;
    bodies[a].position = bodies[a].position + ds * ta;
    bodies[b].position = bodies[b].position - ds * tb;
}

fn get_radius(ct: &ColliderType) -> anyhow::Result<f32> {
    match ct {
        ColliderType::Sphere { radius } => Ok(*radius),
        _ => Err(anyhow::anyhow!(
            "collider type not sphere, could not get radius"
        )),
    }
}
