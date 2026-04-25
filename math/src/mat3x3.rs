
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