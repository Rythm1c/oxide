// _______________________________________________________________________________________________________
// _______________________________________________________________________________________________________
// my home made math library
// got alot of help from the "gabor szauer - hands on c++ game animation programming packt" book
// most of this is just the books code translated to rust with a few changes here and there.
// and https://songho.ca/opengl/ was also pretty helpfull

pub mod mat3;
pub mod mat4;
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
        assert_eq!(vec3(5, 10, 11), vec3(2, 3, 15) + vec3(3, 7, -4));
    }
}
