use super::{mat4::*, quaternion::*, vec3::*};

#[derive(Clone, Debug, Copy, Default)]
pub struct Transform {
    pub translation: Vec3,
    pub scaling: Vec3,
    pub orientation: Quat,
}

impl Transform {
    pub const DEFAULT: Self = Self {
        translation: Vec3::ZERO,
        orientation: Quat::ZERO,
        scaling: Vec3::ONE,
    };

    pub fn new(scaling: Vec3, translation: Vec3, orientation: Quat) -> Self {
        Self {
            translation,
            scaling,
            orientation,
        }
    }

    pub fn lerp(&self, other: &Self, factor: f32) -> Transform {
        Self {
            translation: self.translation.mix(other.translation, factor),
            scaling: self.scaling.mix(other.scaling, factor),
            orientation: self.orientation.nlerp(other.orientation, factor),
        }
    }

    pub fn from_mat(mat: &Mat4) -> Self {
        let mut transform = Self::DEFAULT;

        let translation = Vec3 {
            x: mat.data[0][3],
            y: mat.data[1][3],
            z: mat.data[2][3],
        };

        let orientation = mat.to_quat();
        let d = &mat.data;
        let rot_scale_mat = Mat4 {
            data: [
                [d[0][0], d[0][1], d[0][2], 0.0],
                [d[1][0], d[1][1], d[1][2], 0.0],
                [d[2][0], d[2][1], d[2][2], 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        };
        let inv_rot_mat = orientation.inverse().to_mat();
        let scale_skew_mat = rot_scale_mat * inv_rot_mat;

        let scaling = vec3(
            scale_skew_mat.data[0][0],
            scale_skew_mat.data[1][1],
            scale_skew_mat.data[2][2],
        );

        transform.translation = translation;
        transform.orientation = orientation;
        transform.scaling = scaling;

        transform
    }

    pub fn to_mat(&self) -> Mat4 {
        let mut x = self.orientation * vec3(1.0, 0.0, 0.0);
        let mut y = self.orientation * vec3(0.0, 1.0, 0.0);
        let mut z = self.orientation * vec3(0.0, 0.0, 1.0);

        x = x * self.scaling.x;
        y = y * self.scaling.y;
        z = z * self.scaling.z;

        let p = self.translation;

        Mat4 {
            data: [
                [x.x, y.x, z.x, p.x],
                [x.y, y.y, z.y, p.y],
                [x.z, y.z, z.z, p.z],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn inverse(&self) -> Self {
        let mut inv = Transform::DEFAULT;

        inv.orientation = self.orientation.inverse();

        inv.scaling.x = 1.0 / self.scaling.x;
        inv.scaling.y = 1.0 / self.scaling.y;
        inv.scaling.z = 1.0 / self.scaling.z;

        let inv_trans = -self.translation;
        inv.translation = inv.orientation * (inv.scaling * inv_trans);

        inv
    }

    pub fn combine(&self, rhs: &Self) -> Self {
        let mut out = Transform::DEFAULT;

        out.scaling = self.scaling * rhs.scaling;

        out.orientation = self.orientation * rhs.orientation;
        //mhhhh have no idea what this is
        out.translation = self.orientation * (self.scaling * rhs.translation);

        out.translation = self.translation + out.translation;

        out
    }
}
