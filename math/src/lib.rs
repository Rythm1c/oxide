// _______________________________________________________________________________________________________
// _______________________________________________________________________________________________________
// my home made math library
// got alot of help from the "gabor szauer - hands on c++ game animation programming packt" book
// most of this is just the books code translated to rust with a few changes here and there.
// and https://songho.ca/opengl/ was also pretty helpfull

pub mod mat3x3;
pub mod mat4x4;
pub mod misc;
pub mod quaternion;
pub mod transform;
pub mod vec2;
pub mod vec3;

#[cfg(test)]
mod tests {
    use super::*;
    use vec3::*;

    #[test]
    fn it_works() {
        assert_eq!(vec3(5.0, 10.0, 11.0), vec3(2.0, 3.0, 15.0) + vec3(3.0, 7.0, -4.0));
    }
}
