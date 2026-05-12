use math::{mat3x3::Mat3x3, vec3::Vec3};

pub enum Collider {
    Sphere(SphereCollider),
}

impl Collider {
    pub fn center_of_mass(&self) -> Vec3 {
        match self {
            Collider::Sphere(..) => Vec3::ZERO,
        }
    }

    pub fn inertia_tensor(&self) -> Mat3x3 {
        match self {
            Collider::Sphere(sphere) => {
                let i = (2.0 / 5.0) * sphere.radius * sphere.radius;
                Mat3x3::new(
                    i, 0.0, 0.0,
                    0.0, i, 0.0,
                    0.0, 0.0, i,
                )
            }
        }
    }
}

pub struct SphereCollider {
    pub radius: f32,
}

impl SphereCollider {
    pub fn new(radius: f32) -> Self {
        Self { radius }
    }
}
