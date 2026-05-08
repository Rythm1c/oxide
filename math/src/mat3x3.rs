use std::ops::Mul;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Mat3x3 {
    pub data: [[f32; 3]; 3],
}

pub fn mat3x3(
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

    pub fn transpose(&self) -> Mat3x3 {
        Mat3x3 {
            data: [
                [self.data[0][0], self.data[1][0], self.data[2][0]],
                [self.data[0][1], self.data[1][1], self.data[2][1]],
                [self.data[0][2], self.data[1][2], self.data[2][2]],
            ],
        }
    }
}

// helper for 3x3 matrix multiplication
fn rxc(r: usize, c: usize, m1: &Mat3x3, m2: &Mat3x3) -> f32 {
    let a = m1.data[r][0] * m2.data[0][c];
    let b = m1.data[r][1] * m2.data[1][c];
    let c = m1.data[r][2] * m2.data[2][c];
    return a + b + c;
}

impl Mul<Mat3x3> for Mat3x3{
    type Output = Mat3x3;
    fn mul(self, rhs: Self) -> Self::Output {
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
