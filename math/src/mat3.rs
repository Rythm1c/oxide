//comming soon...

// A three by three matrix
pub struct Mat3 {
    pub data: [[f32; 3]; 3],
}

pub fn mat3(
    xx: f32,
    xy: f32,
    xz: f32,

    yx: f32,
    yy: f32,
    yz: f32,

    zx: f32,
    zy: f32,
    zz: f32,
) -> Mat3 {
    Mat3 {
        data: [
            [xx, xy, xz],
            [yx, yy, yz],
            [zx, zy, zz],
          
        ],
    }
}

impl Mat3{
    pub fn identity()->Self{
        Mat3 {
            data: [
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn transpose(&self) -> Mat3 {
        Mat3 {
            data: [
                [self.data[0][0], self.data[1][0], self.data[2][0]],
                [self.data[0][1], self.data[1][1], self.data[2][1]],
                [self.data[0][2], self.data[1][2], self.data[2][2]],
            ],
        }
    }

    
}