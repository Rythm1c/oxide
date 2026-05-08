use std::ops::Mul;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Mat2x2{
    data: [[f32; 2]; 2]
}

impl Mat2x2 {

    pub fn identity() -> Self {
        Self{
            data: [
                [1.0, 0.0],
                [0.0, 0.0],
            ]
        }
    }

    pub fn new(
        xx: f32, xy: f32,
        yx: f32, yy: f32)
        -> Self {
        Self {
            data: [
                [xx, xy],
                [yx, yy]
            ]
        }
    }

    pub fn transpose(&self) -> Self {
        let d = &self.data;
        Self {
            data: [
                [d[0][0], d[1][0]],
                [d[0][1], d[1][1]]
        ] }
    }

    pub fn determinant(&self) -> f32 {
        let d = &self.data;

        let a = d[0][0] * d[1][1];
        let b = d[0][1] * d[1][0];

        a - b
    }

    pub fn inverse(&self) -> Self {
        let d = &self.data;
        let det = self.determinant();

        (1.0 / det ) * Self::new(d[1][1], -d[0][1], -d[1][0], d[0][0])
    }

}

impl Mul<f32> for Mat2x2{
    type Output = Mat2x2;
    fn mul(self, rhs: f32) -> Self::Output {
        let  d = &self.data;
        Self {
            data: [
                [rhs * d[0][0], rhs * d[0][1]],
                [rhs * d[1][0], rhs * d[1][1]]
            ]
        }
    }
}

impl Mul<Mat2x2> for f32{
    type Output = Mat2x2;
    fn mul(self, rhs: Mat2x2) -> Self::Output {
        rhs * self
    }
}
