use std::ops::Mul;

use crate::vec3::Vec3;

use super::quaternion::Quat;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Mat3x3 {
    pub data: [[f32; 3]; 3],
}

impl Mat3x3{

    pub fn identity()->Self{
        Mat3x3 {
            data: [
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn new(
        xx: f32, xy: f32, xz: f32,
        yx: f32, yy: f32, yz: f32,
        zx: f32, zy: f32, zz: f32,
    ) -> Mat3x3 {
        Mat3x3 {
            data: [
                [xx, xy, xz],
                [yx, yy, yz],
                [zx, zy, zz],
            ],
        }
    }

    pub fn minor(&self, r: u32, c: u32) -> f32 {
        let d = &self.data;
        match (r, c) {
            (0, 0) => d[1][1] * d[2][2] - d[1][2] * d[2][1],
            (0, 1) => d[1][0] * d[2][2] - d[1][2] * d[2][0],
            (0, 2) => d[1][0] * d[2][1] - d[1][1] * d[2][0],

            (1, 0) => d[0][1] * d[2][2] - d[0][2] * d[2][1],
            (1, 1) => d[0][0] * d[2][2] - d[0][2] * d[2][0],
            (1, 2) => d[0][0] * d[2][1] - d[0][1] * d[2][0],

            (2, 0) => d[0][1] * d[1][2] - d[0][2] * d[1][1],
            (2, 1) => d[0][0] * d[1][2] - d[0][2] * d[1][0],
            (2, 2) => d[0][0] * d[1][1] - d[0][1] * d[1][0],
            _ => panic!("index out of range"),
        }
    }

    pub fn cofactor(&self, r: u32, c: u32) -> f32 {
        // (-1.0) ^ (row + col)
        let sign = if (r + c) % 2 == 0 {1.0} else {-1.0};

        sign * self.minor(r, c)
    }

    pub fn determinant(&self) -> f32 {
        let a = self.data[0][0] * self.cofactor(0, 0);
        let b = self.data[0][1] * self.cofactor(0, 1);
        let c = self.data[0][2] * self.cofactor(0, 2);

        a + b + c
    }

    pub fn adjugate(&self) -> Self {
        //Cof (M[i, j]) = Minor(M[i, j]]) * pow(-1, i + j)
        //let m = &self.data;
        let mut cofactor = Self::identity();
        for i in 0..3 {
            for j in 0..3{
                cofactor.data[i][j] = self.cofactor(i as u32, j as u32);
            }
        }

        cofactor.transpose()
    }

    pub fn inverse(&self) -> Self {
        let det = self.determinant();

        if det.abs() < f32::EPSILON {
            return Self::identity();
        }
        let adj = self.adjugate();

        adj * (1.0 / det)
    }

    pub fn transpose(&self) -> Mat3x3 {
        let d = &self.data;

        Mat3x3 {
            data: [
                [d[0][0], d[1][0], d[2][0]],
                [d[0][1], d[1][1], d[2][1]],
                [d[0][2], d[1][2], d[2][2]],
            ],
        }
    }

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

impl Mul<f32> for Mat3x3 {
    type Output = Mat3x3;
    fn mul(self, rhs: f32) -> Self::Output {
        let mut output = Mat3x3::identity();

        for i in 0..3 {
            for j in 0..3 {
                output.data[i][j] = rhs * self.data[i][j];
            }
        }

        output
    }
}

impl Mul<Mat3x3> for f32 {
    type Output = Mat3x3;
    fn mul(self, rhs: Mat3x3) -> Self::Output {
        rhs * self
    }
}

impl Mul<Mat3x3> for Mat3x3{
    type Output = Mat3x3;
    fn mul(self, rhs: Self) -> Self::Output {
        // helper for 3x3 matrix multiplication
        let rxc= |r: usize, c: usize, m1: &Mat3x3, m2: &Mat3x3| -> f32 {
            let a = m1.data[r][0] * m2.data[0][c];
            let b = m1.data[r][1] * m2.data[1][c];
            let c = m1.data[r][2] * m2.data[2][c];
            return a + b + c;
        };

        let xx = rxc(0, 0, &self, &rhs);
        let xy = rxc(0, 1, &self, &rhs);
        let xz = rxc(0, 2, &self, &rhs);

        let yx = rxc(1, 0, &self, &rhs);
        let yy = rxc(1, 1, &self, &rhs);
        let yz = rxc(1, 2, &self, &rhs);

        let zx = rxc(2, 0, &self, &rhs);
        let zy = rxc(2, 1, &self, &rhs);
        let zz = rxc(2, 2, &self, &rhs);
        Mat3x3{
            data: [
                [xx, xy, xz],
                [yx, yy, yz],
                [zx, zy, zz]
            ]
        }

    }
}

impl Mul<Vec3> for Mat3x3 {
    type Output = Vec3;
    fn mul(self, rhs: Vec3) -> Self::Output {
        //helper function
        let rxv = |m: &[[f32; 3]; 3],v: &Vec3, r: usize|-> f32 {
            let a = m[r][0] * v.x;
            let b = m[r][1] * v.y;
            let c = m[r][2] * v.z;

            a + b + c
        };

        Vec3 {
            x: rxv(&self.data, &rhs, 0),
            y: rxv(&self.data, &rhs, 1),
            z: rxv(&self.data, &rhs, 2),
        }
    }
}
