use std::ops::Mul;

use crate::mat2x2::Mat2x2;

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
        let mut arr :Vec<f32> = Vec::with_capacity(4);

        for i in 0..3 {
            if i == r {
                continue;
            }

            for j in 0..3{
                if j == c {
                    continue;
                }

                arr.push(d[i as usize][j as usize]);
            }
        }

        Mat2x2::new(
            arr[0], arr[1],
            arr[2], arr[3])
        .determinant()
    }

    pub fn cofactor(&self, r: u32, c: u32) -> f32 {
        // +1 because we start with index 0
        let power: u32 = r + 1 + c + 1;
        let sign = (-1i32).pow(power) as f32;

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

    pub fn inverse(self) -> Self {
        let det = self.determinant();

        if det == 0.0 {
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
