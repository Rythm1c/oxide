use super::misc::*;

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    pub const ONE: Self = Self {
        x: 1.0,
        y: 1.0,
        z: 1.0,
    };

    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    /// get 3D vector form array
    pub fn from(a: &[f32; 3]) -> Self {
        Self {
            x: a[0],
            y: a[1],
            z: a[2],
        }
    }
    pub fn to_array(&self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }
    // vector length
    pub fn len(&self) -> f32 {
        (self.x.powf(2.0) + self.y.powf(2.0) + self.z.powf(2.0)).sqrt()
    }

    // get the vector normalized
    pub fn unit(&self) -> Self {
        let inverse_legth = 1.0 / self.len();
        Self {
            x: self.x * inverse_legth,
            y: self.y * inverse_legth,
            z: self.z * inverse_legth,
        }
    }
    /// linear interpolation between 2 vectors
    pub fn mix(&self, other: Self, c: f32) -> Self {
        (*self * (1.0 - c)) + (other * c)
    }

    pub fn max(&self, other: Self) -> Self {
        Self {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
            z: self.z.max(other.z),
        }
    }

    pub fn min(&self, other: Self) -> Self {
        Self {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
            z: self.z.min(other.z),
        }
    }
}

pub fn vec3(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}
// dot product with another vector
pub fn dot(v1: &Vec3, v2: &Vec3) -> f32 {
    (v1.x * v2.x) + (v1.y * v2.y) + (v1.z * v2.z)
}
// reflect a vector around a normal
pub fn reflect(incident: &Vec3, normal: &Vec3) -> Vec3 {
    *incident - 2.0 * dot(incident, normal) * *normal
}
//get the cross product between two vectors
pub fn cross(v1: &Vec3, v2: &Vec3) -> Vec3 {
    Vec3::new(
        v1.y * v2.z - v1.z * v2.y,
        v1.z * v2.x - v1.x * v2.z,
        v1.x * v2.y - v1.y * v2.x,
    )
}

pub fn clamp_vec3(v: &Vec3, min: &Vec3, max: &Vec3) -> Vec3 {
    Vec3::new(
        clamp(v.x, min.x, max.x),
        clamp(v.y, min.y, max.y),
        clamp(v.z, min.z, max.z),
    )
}
use std::ops::*;
impl Add for Vec3 {
    type Output = Vec3;
    fn add(self, rhs: Self) -> Self {
        Self {
            x: rhs.x + self.x,
            y: rhs.y + self.y,
            z: rhs.z + self.z,
        }
    }
}
impl Sub for Vec3 {
    type Output = Vec3;
    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Mul<f32> for Vec3 {
    type Output = Vec3;
    fn mul(self, rhs: f32) -> Self {
        Self {
            x: rhs * self.x,
            y: rhs * self.y,
            z: rhs * self.z,
        }
    }
}
impl Mul<Vec3> for f32 {
    type Output = Vec3;
    fn mul(self, rhs: Vec3) -> Self::Output {
        Vec3 {
            x: rhs.x * self,
            y: rhs.y * self,
            z: rhs.z * self,
        }
    }
}
impl Mul<Vec3> for Vec3 {
    type Output = Self;
    fn mul(self, rhs: Vec3) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}
impl Div<f32> for Vec3 {
    type Output = Vec3;
    fn div(self, rhs: f32) -> Self {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl Neg for Vec3 {
    type Output = Vec3;
    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}
