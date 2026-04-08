use super::misc;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

pub fn vec2(_x: f32, _y: f32) -> Vec2 {
    Vec2 { x: _x, y: _y }
}

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub const ONE: Self = Self { x: 1.0, y: 1.0 };

    pub fn from(a: &[f32; 2]) -> Self {
        Self { x: a[0], y: a[1] }
    }

    pub fn step(&self, other: &Self) -> [i32; 2] {
        [misc::step(self.x, other.x), misc::step(self.y, other.y)]
    }
}
