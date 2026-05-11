use math::{mat3x3::Mat3x3, vec3::Vec3};


pub enum ColliderType {
    Sphere{radius: f32},
    //Cube,
    //Plane
}

impl ColliderType {
    pub fn sphere(radius: f32) -> Self {
        ColliderType::Sphere { radius }
    }

    pub fn get_center_of_mass(&self) -> Vec3 {
        match self {
            ColliderType::Sphere { .. } => Vec3::ZERO,
        }
    }

    pub fn get_inverse_inertia_tensor(&self) -> Mat3x3{
        match self {
            ColliderType::Sphere { .. } => Mat3x3::identity(),
        }
    }
}
