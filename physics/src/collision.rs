use crate::{collider::ColliderType, rigidbody::RigidBody};

pub fn test_sphere_shere_intersection(a: &RigidBody, b: &RigidBody) -> anyhow::Result<bool> {
    let ab = a.position - b.position;

    let ra = get_radius(a.collider_type.as_ref().unwrap())?;
    let rb = get_radius(&b.collider_type.as_ref().unwrap())?;
    let rab = ra + rb;

    let ab_sqrd = ab.len_sqrd();
    if ab_sqrd <= (rab * rab) {
        return Ok(true);
    }

    Ok(false)
}

fn get_radius(ct: &ColliderType) -> anyhow::Result<f32> {
    match ct {
        ColliderType::Sphere { radius } => Ok(*radius),
        _ => Err(anyhow::anyhow!(
            "collider type not sphere, could not get radius"
        )),
    }
}
