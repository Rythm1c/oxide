pub const PIE: f32 = 3.1415927;

pub fn radians(v: f32) -> f32 {
    v * (PIE / 180.0)
}
pub fn minimum(a: f32, b: f32) -> f32 {
    if a < b {
        a
    } else {
        b
    }
}
pub fn maximum(a: f32, b: f32) -> f32 {
    if a > b {
        a
    } else {
        b
    }
}

pub fn clamp(v: f32, min: f32, max: f32) -> f32 {
    maximum(min, minimum(v, max))
}

pub fn step(a: f32, b: f32) -> i32 {
    if b < a {
        return 0;
    } else {
        return 1;
    }
}
