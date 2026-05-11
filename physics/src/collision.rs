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

    pub separation_distance: f32,
    pub time_of_impact: f32,

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

            separation_distance: 0.0,
            time_of_impact: 0.0,
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

        contact.pt_a_world_space = bodies[a].position - contact.normal * ra;
        contact.pt_b_world_space = bodies[b].position + contact.normal * rb;
        return Ok(contact);
    }

    contact.collision = false;
    Ok(contact)
}

pub fn resolve_contact(contact: &Contact, bodies: &mut Vec<RigidBody>) {
    if !contact.collision {
        return;
    }

    bodies[contact.body_a].velocity = Vec3::ZERO;
    bodies[contact.body_b].velocity = Vec3::ZERO;
}

fn get_radius(ct: &ColliderType) -> anyhow::Result<f32> {
    match ct {
        ColliderType::Sphere { radius } => Ok(*radius),
        _ => Err(anyhow::anyhow!(
            "collider type not sphere, could not get radius"
        )),
    }
}
