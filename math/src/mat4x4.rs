// got alot of help from the "gabor szauer - hands on c++ game animation programming packt" book
// most of this is just the books code translated to rust with a few changes here and there.
// and https://songho.ca/opengl/ was also pretty helpfull

#![allow(dead_code)]
use super::{misc::*, quaternion::Quat, vec3::*};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Mat4x4 {
    pub data: [[f32; 4]; 4],
}

pub fn mat4x4(
    xx: f32, xy: f32, xz: f32, xw: f32,
    yx: f32, yy: f32, yz: f32, yw: f32,
    zx: f32, zy: f32, zz: f32, zw: f32,
    wx: f32, wy: f32, wz: f32, ww: f32,
) -> Mat4x4 {
    Mat4x4 {
        data: [
            [xx, xy, xz, xw],
            [yx, yy, yz, yw],
            [zx, zy, zz, zw],
            [wx, wy, wz, ww],
        ],
    }
}

impl Mat4x4 {
    pub const IDENTITY: Self = Self {
        data: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    };

    pub fn from(values: &[[f32; 4]; 4]) -> Self {
        Self { data: *values }
    }

    pub fn flattended(&self) -> [f32; 16] {
        let data = self.data.as_flattened();

        [
            data[0],  data[1],  data[2],  data[3], 
            data[4],  data[5],  data[6],  data[7], 
            data[8],  data[9],  data[10], data[11],
            data[12], data[13], data[14], data[15],
        ]
    }
    /// changes signs past 180 degrees  
    /// not sure why though
    pub fn to_quat(&self) -> Quat {
        let data = &self.data;

        let s = 0.5 * (1.0 + data[0][0] + data[1][1] + data[2][2]).sqrt();
        if s > 0.0 {
            let coeff = 1.0 / (4.0 * s);
            let x = coeff * (data[2][1] - data[1][2]);
            let y = coeff * (data[0][2] - data[2][0]);
            let z = coeff * (data[1][0] - data[0][1]);
            return Quat { x, y, z, s };
        }
        let x = 0.5 * (1.0 + data[0][0] - data[1][1] - data[2][2]).sqrt();
        if x > 0.0 {
            let coeff = 1.0 / (4.0 * x);
            let y = coeff * (data[0][1] + data[1][0]);
            let z = coeff * (data[0][2] + data[2][0]);
            let s = coeff * (data[2][1] - data[1][2]);
            return Quat { x, y, z, s };
        }
        let y = 0.5 * (1.0 - data[0][0] + data[1][1] - data[2][2]).sqrt();
        if y > 0.0 {
            let coeff = 1.0 / (4.0 * y);
            let x = coeff * (data[0][1] + data[1][0]);
            let z = coeff * (data[1][2] + data[2][1]);
            let s = coeff * (data[0][2] - data[2][0]);
            return Quat { x, y, z, s };
        }
        // if all else fails just use z
        let z = 0.5 * (1.0 - data[0][0] - data[1][1] + data[2][2]).sqrt();
        let coeff = 1.0 / (4.0 * z);
        let x = coeff * (data[0][2] + data[2][0]);
        let y = coeff * (data[1][2] + data[2][1]);
        let s = coeff * (data[1][0] - data[0][1]);
        return Quat { x, y, z, s };
    }
}
use std::fmt::Display;
use std::ops::*;
impl Mul<Mat4x4> for f32 {
    type Output = Mat4x4;
    fn mul(self, rhs: Mat4x4) -> Self::Output {
        Mat4x4 {
            data: [
                [
                    rhs.data[0][0] * self,
                    rhs.data[0][1] * self,
                    rhs.data[0][2] * self,
                    rhs.data[0][3] * self,
                ],
                [
                    rhs.data[1][0] * self,
                    rhs.data[1][1] * self,
                    rhs.data[1][2] * self,
                    rhs.data[1][3] * self,
                ],
                [
                    rhs.data[2][0] * self,
                    rhs.data[2][1] * self,
                    rhs.data[2][2] * self,
                    rhs.data[2][3] * self,
                ],
                [
                    rhs.data[3][0] * self,
                    rhs.data[3][1] * self,
                    rhs.data[3][2] * self,
                    rhs.data[3][3] * self,
                ],
            ],
        }
    }
}

/// matrix multiplication helper.
/// multiply corresponding row and column elements
fn c_r(row: usize, column: usize, m1: &Mat4x4, m2: &Mat4x4) -> f32 {
    let v1 = m1.data[row][0] * m2.data[0][column];
    let v2 = m1.data[row][1] * m2.data[1][column];
    let v3 = m1.data[row][2] * m2.data[2][column];
    let v4 = m1.data[row][3] * m2.data[3][column];

    v1 + v2 + v3 + v4
}

impl Mul<Mat4x4> for Mat4x4 {
    type Output = Mat4x4;
    fn mul(self, rhs: Mat4x4) -> Self::Output {
        Self {
            data: [
                [
                    c_r(0, 0, &self, &rhs),
                    c_r(0, 1, &self, &rhs),
                    c_r(0, 2, &self, &rhs),
                    c_r(0, 3, &self, &rhs),
                ],
                [
                    c_r(1, 0, &self, &rhs),
                    c_r(1, 1, &self, &rhs),
                    c_r(1, 2, &self, &rhs),
                    c_r(1, 3, &self, &rhs),
                ],
                [
                    c_r(2, 0, &self, &rhs),
                    c_r(2, 1, &self, &rhs),
                    c_r(2, 2, &self, &rhs),
                    c_r(2, 3, &self, &rhs),
                ],
                [
                    c_r(3, 0, &self, &rhs),
                    c_r(3, 1, &self, &rhs),
                    c_r(3, 2, &self, &rhs),
                    c_r(3, 3, &self, &rhs),
                ],
            ],
        }
    }
}
impl Mul<f32> for Mat4x4 {
    type Output = Mat4x4;
    fn mul(self, rhs: f32) -> Self::Output {
        let mut result = Mat4x4::IDENTITY;

        for i in 0..4 {
            for j in 0..4 {
                result.data[i][j] = self.data[i][j] * rhs;
            }
        }

        result
    }
}

impl Display for Mat4x4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "
            [{}, {}, {}, {}]\n
            [{}, {}, {}, {}]\n
            [{}, {}, {}, {}]\n
            [{}, {}, {}, {}]",
            self.data[0][0],
            self.data[0][1],
            self.data[0][2],
            self.data[0][3],
            self.data[1][0],
            self.data[1][1],
            self.data[1][2],
            self.data[1][3],
            self.data[2][0],
            self.data[2][1],
            self.data[2][2],
            self.data[2][3],
            self.data[3][0],
            self.data[3][1],
            self.data[3][2],
            self.data[3][3],
        )
    }
}

fn m4_3x3minor(
    x: &[[f32; 4]; 4],
    c0: usize,
    c1: usize,
    c2: usize,
    r0: usize,
    r1: usize,
    r2: usize,
) -> f32 {
    let a = x[r0][c0] * (x[r1][c1] * x[r2][c2] - x[r2][c1] * x[r1][c2]);
    let b = x[r0][c1] * (x[r1][c0] * x[r2][c2] - x[r2][c0] * x[r1][c2]);
    let c = x[r0][c2] * (x[r1][c0] * x[r2][c1] - x[r2][c0] * x[r1][c1]);

    a - b + c
}

fn determinant(m: &Mat4x4) -> f32 {
    let a = m.data[0][0] * m4_3x3minor(&m.data, 1, 2, 3, 1, 2, 3);
    let b = m.data[0][1] * m4_3x3minor(&m.data, 0, 2, 3, 1, 2, 3);
    let c = m.data[0][2] * m4_3x3minor(&m.data, 0, 1, 3, 1, 2, 3);
    let d = m.data[0][3] * m4_3x3minor(&m.data, 0, 1, 2, 1, 2, 3);

    a - b + c - d
}

fn adjugate(m: &Mat4x4) -> Mat4x4 {
    //Cof (M[i, j]) = Minor(M[i, j]]) * pow(-1, i + j)
    let mut cofactor = Mat4x4::IDENTITY;
    cofactor.data[0][0] =  m4_3x3minor(&m.data, 1, 2, 3, 1, 2, 3);
    cofactor.data[1][0] = -m4_3x3minor(&m.data, 1, 2, 3, 0, 2, 3);
    cofactor.data[2][0] =  m4_3x3minor(&m.data, 1, 2, 3, 0, 1, 3);
    cofactor.data[3][0] = -m4_3x3minor(&m.data, 1, 2, 3, 0, 1, 2);
    cofactor.data[0][1] = -m4_3x3minor(&m.data, 0, 2, 3, 1, 2, 3);
    cofactor.data[1][1] =  m4_3x3minor(&m.data, 0, 2, 3, 0, 2, 3);
    cofactor.data[2][1] = -m4_3x3minor(&m.data, 0, 2, 3, 0, 1, 3);
    cofactor.data[3][1] =  m4_3x3minor(&m.data, 0, 2, 3, 0, 1, 2);
    cofactor.data[0][2] =  m4_3x3minor(&m.data, 0, 1, 3, 1, 2, 3);
    cofactor.data[1][2] = -m4_3x3minor(&m.data, 0, 1, 3, 0, 2, 3);
    cofactor.data[2][2] =  m4_3x3minor(&m.data, 0, 1, 3, 0, 1, 3);
    cofactor.data[3][2] = -m4_3x3minor(&m.data, 0, 1, 3, 0, 1, 2);
    cofactor.data[0][3] = -m4_3x3minor(&m.data, 0, 1, 2, 1, 2, 3);
    cofactor.data[1][3] =  m4_3x3minor(&m.data, 0, 1, 2, 0, 2, 3);
    cofactor.data[2][3] = -m4_3x3minor(&m.data, 0, 1, 2, 0, 1, 3);
    cofactor.data[3][3] =  m4_3x3minor(&m.data, 0, 1, 2, 0, 1, 2);

    transpose(&cofactor)
}

pub fn inverse(m: &Mat4x4) -> Mat4x4 {
    let det = determinant(m);

    if det == 0.0 {
        return Mat4x4::IDENTITY;
    }
    let adj = adjugate(m);

    adj * (1.0 / det)
}

pub fn transpose(m: &Mat4x4) -> Mat4x4 {
    Mat4x4 {
        data: [
            [m.data[0][0], m.data[1][0], m.data[2][0], m.data[3][0]],
            [m.data[0][1], m.data[1][1], m.data[2][1], m.data[3][1]],
            [m.data[0][2], m.data[1][2], m.data[2][2], m.data[3][2]],
            [m.data[0][3], m.data[1][3], m.data[2][3], m.data[3][3]],
        ],
    }
}

pub fn translate(p: &Vec3) -> Mat4x4 {
    Mat4x4 {
        data: [
            [1.0, 0.0, 0.0, p.x],
            [0.0, 1.0, 0.0, p.y],
            [0.0, 0.0, 1.0, p.z],
            [0.0, 0.0, 0.0, 1.0],
        ],
    }
}
pub fn scale(s: &Vec3) -> Mat4x4 {
    Mat4x4 {
        data: [
            [s.x, 0.0, 0.0, 0.0],
            [0.0, s.y, 0.0, 0.0],
            [0.0, 0.0, s.z, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    }
}

//rotation matrices using euler angles
///rotation around the x-axis using specified angle
pub fn rotation_x(angle: f32) -> Mat4x4 {
    let yy = radians(angle).cos();
    let yz = -radians(angle).sin();

    let zy = radians(angle).sin();
    let zz = radians(angle).cos();

    return Mat4x4 {
        data: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0,  yy,  yz, 0.0],
            [0.0,  zy,  zz, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    };
}

///rotation around the y-axis using specified angle
pub fn rotation_y(angle: f32) -> Mat4x4 {
    let xx = radians(angle).cos();
    let xz = radians(angle).sin();

    let zx = -radians(angle).sin();
    let zz = radians(angle).cos();

    return Mat4x4 {
        data: [
            [ xx, 0.0,  xz, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [ zx, 0.0,  zz, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    };
}

///rotation around the Z-axis using specified angle
pub fn rotation_z(angle: f32) -> Mat4x4 {
    let xx = radians(angle).cos();
    let xy = -radians(angle).sin();

    let yx = radians(angle).sin();
    let yy = radians(angle).cos();

    return Mat4x4 {
        data: [
            [ xx,  xy, 0.0, 0.0],
            [ yx,  yy, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    };
}

/// for camera rotation
pub fn look_at(eye: Vec3, front: Vec3, up: Vec3) -> Mat4x4 {
    // camera direction
    let cd = (eye - front).unit();
    // get right vector
    let cr = cross(&up, &cd).unit();
    // get up vector
    let cu = cross(&cd, &cr).unit();

    // translation vector
    let xw = -(eye.x * cr.x) - (eye.y * cr.y) - (eye.z * cr.z);
    let yw = -(eye.x * cu.x) - (eye.y * cu.y) - (eye.z * cu.z);
    let zw = -(eye.x * cd.x) - (eye.y * cd.y) - (eye.z * cd.z);

    Mat4x4 {
        data: [
            [cr.x, cr.y, cr.z,  xw],
            [cu.x, cu.y, cu.z,  yw],
            [cd.x, cd.y, cd.z,  zw],
            [ 0.0,  0.0,  0.0, 1.0],
        ],
    }
}
/// l: left, r: right, n: near, f: far, t: top, b: bottom  
/// create a clipping volume from sepcified distances
pub fn frustrum(l: f32, r: f32, t: f32, b: f32, n: f32, f: f32) -> Mat4x4 {
    let xx = (2.0 * n) / (r - l);
    let xz = (r + l) / (r - l);

    let yy = (2.0 * n) / (t - b);
    let yz = (t + b) / (t - b);

    let zz = -(f + n) / (f - n);
    let zw = (-2.0 * f * n) / (f - n);

    Mat4x4 {
        data: [
            [ xx, 0.0,   xz, 0.0],
            [0.0,  yy,   yz, 0.0],
            [0.0, 0.0,   zz,  zw],
            [0.0, 0.0, -1.0, 0.0],
        ],
    }
}

pub fn orthogonal(r: f32, l: f32, t: f32, b: f32, n: f32, f: f32) -> Mat4x4 {
    let xx = 2.0 / (r - l);
    let xw = -(r + l) / (r - l);

    let yy = 2.0 / (t - b);
    let yw = -(t + b) / (t - b);

    let zz = -2.0 / (f - n);
    let zw = -(n + f) / (f - n);
    Mat4x4 {
        data: [
            [ xx, 0.0, 0.0,  xw],
            [0.0,  yy, 0.0,  yw],
            [0.0, 0.0,  zz,  zw],
            [0.0, 0.0, 0.0, 1.0],
        ],
    }
}

pub fn perspective(fov: f32, aspect_ratio: f32, near: f32, far: f32) -> Mat4x4 {
    let tangent = f32::tan(radians(fov / 2.0));
    let top     = near * tangent;
    let right   = top * aspect_ratio;

    frustrum(-right, right, top, -top, near, far)
}
