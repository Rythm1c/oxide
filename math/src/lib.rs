// _______________________________________________________________________________________________________
// _______________________________________________________________________________________________________
// my home made math library
// got alot of help from the "gabor szauer - hands on c++ game animation programming packt" book
// most of this is just the books code translated to rust with a few changes here and there.
// and https://songho.ca/opengl/ was also pretty helpfull

pub mod mat2x2;
pub mod mat3x3;
pub mod mat4x4;
pub mod quaternion;
pub mod transform;
pub mod vec2;
pub mod vec3;

#[cfg(test)]
mod tests {
    use super::*;
    use mat2x2::Mat2x2;
    use mat3x3::Mat3x3;
    use mat4x4::Mat4x4;
    use vec3::Vec3;

    #[test]
    fn vec3_addition() {
        assert_eq!(
            Vec3(5.0, 10.0, 11.0),
            Vec3(2.0, 3.0, 15.0) + Vec3(3.0, 7.0, -4.0)
        );
    }
}
