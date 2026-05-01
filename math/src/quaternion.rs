//----------------------------------------------------------------------------------------------------
//----------------------------------------------------------------------------------------------------
// home made quaternion math lib cause i have a big ego.
// "john vince - quaternions for for computer graphics" was a massive help along with
// "gabor szauer - hands on c++ game animation programming packt", both great books.

use super::{mat4x4::*, vec3::*};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub s: f32,
}

pub fn quat(x: f32, y: f32, z: f32, s: f32) -> Quat {
    Quat { x, y, z, s }
}

impl Quat {
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        s: 1.0,
    };

    pub fn new(x: f32, y: f32, z: f32, s: f32) -> Self {
        Self { x, y, z, s }
    }
    /// get quaternion from array
    pub fn from(a: &[f32; 4]) -> Self {
        quat(a[0], a[1], a[2], a[3])
    }
    pub fn to_array(&self) -> [f32; 4] {
        [self.x, self.y, self.z, self.s]
    }
    /// halves the angle and creates a quaternion from it and the specified axis  
    /// and also axis is normalized so no worries  
    /// resulting quaternion intended to be used with 'to_mat' function
    pub fn create(angle: f32, axis: Vec3) -> Self {
        let s = f32::sin(radians(angle / 2.0));
        let c = f32::cos(radians(angle / 2.0));

        let unit_axis = Vec3::unit(&axis);

        let x = s * unit_axis.x;
        let y = s * unit_axis.y;
        let z = s * unit_axis.z;
        let s = c;

        Self { x, y, z, s }
    }

    pub fn rotation_x(angle: f32) -> Self {
        Self::create(angle, Vec3::X)
    }

    pub fn rotation_y(angle: f32) -> Self {
        Self::create(angle, Vec3::Y)
    }

    pub fn rotation_z(angle: f32) -> Self {
        Self::create(angle, Vec3::Z)
    }

    pub fn norm(&self) -> f32 {
        let x2 = f32::powf(self.x, 2.0);
        let y2 = f32::powf(self.y, 2.0);
        let z2 = f32::powf(self.z, 2.0);
        let s2 = f32::powf(self.s, 2.0);
        f32::sqrt(x2 + y2 + z2 + s2)
    }

    pub fn unit(&self) -> Self {
        let coeff = 1.0 / self.norm();

        Self {
            x: (coeff * self.x),
            y: (coeff * self.y),
            z: (coeff * self.z),
            s: (coeff * self.s),
        }
    }

    pub fn conjugate(&self) -> Self {
        Self {
            x: (-self.x),
            y: (-self.y),
            z: (-self.z),
            s: (self.s),
        }
    }

    pub fn inverse(&self) -> Self {
        let len_sq = self.x * self.x + self.y * self.y + self.z * self.z + self.s * self.s;
        let inv_len = 1.0 / len_sq;
        self.conjugate() * inv_len
    }

    pub fn dot(&self, rhs: &Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z + self.s * rhs.s
    }

    pub fn nlerp(&self, other: Self, c: f32) -> Quat {
        (*self + (other - *self) * c).unit()
    }

    pub fn axis(&self) -> Vec3 {
        vec3(self.x, self.y, self.z)
    }

    /// rotate around a specified axis
    /// creates a rotation matrix from a quaternion
    pub fn to_mat(&self) -> Mat4x4 {
        let x2 = f32::powf(self.x, 2.0);
        let y2 = f32::powf(self.y, 2.0);
        let z2 = f32::powf(self.z, 2.0);
        // first row
        let xx = 1.0 - 2.0 * (y2 + z2);
        let xy = 2.0 * (self.x * self.y - self.s * self.z);
        let xz = 2.0 * (self.x * self.z + self.s * self.y);
        // second row
        let yx = 2.0 * (self.x * self.y + self.s * self.z);
        let yy = 1.0 - 2.0 * (x2 + z2);
        let yz = 2.0 * (self.y * self.z - self.s * self.x);
        // third row
        let zx = 2.0 * (self.x * self.z - self.s * self.y);
        let zy = 2.0 * (self.y * self.z + self.s * self.x);
        let zz = 1.0 - 2.0 * (x2 + y2);

        Mat4x4 {
            data: [
                [ xx,  xy,  xz, 0.0],
                [ yx,  yy,  yz, 0.0],
                [ zx,  zy,  zz, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }
}

use std::ops::*;

use super::misc::radians;
impl Sub for Quat {
    type Output = Quat;
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
            s: self.s - rhs.s,
        }
    }
}

impl Add for Quat {
    type Output = Quat;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
            s: self.s + rhs.s,
        }
    }
}

impl Neg for Quat {
    type Output = Quat;
    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            s: -self.s,
        }
    }
}
impl Mul<f32> for Quat {
    type Output = Quat;
    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
            s: self.s * rhs,
        }
    }
}
impl Mul<Quat> for f32 {
    type Output = Quat;
    fn mul(self, rhs: Quat) -> Self::Output {
        Quat {
            x: self * rhs.x,
            y: self * rhs.y,
            z: self * rhs.z,
            s: self * rhs.s,
        }
    }
}
impl Mul<Vec3> for Quat {
    type Output = Vec3;
    /// same as  
    /// r = (q * v' * q^-1).xyz
    fn mul(self, rhs: Vec3) -> Self::Output {
        let a = self.axis() * 2.0 * dot(&self.axis(), &rhs);
        let b = rhs * (self.s * self.s - dot(&self.axis(), &self.axis()));
        let c = cross(self.axis(), rhs) * 2.0 * self.s;

        a + b + c
    }
}

impl Mul<Quat> for Quat {
    type Output = Quat;
    fn mul(self, rhs: Quat) -> Self::Output {
        Self {
            x: self.s * rhs.x + self.x * rhs.s + self.y * rhs.z - self.z * rhs.y,
            y: self.s * rhs.y + self.y * rhs.s + self.z * rhs.x - self.x * rhs.z,
            z: self.s * rhs.z + self.z * rhs.s + self.x * rhs.y - self.y * rhs.x,
            s: self.s * rhs.s - self.x * rhs.x - self.y * rhs.y - self.z * rhs.z,
        }
    }
}
