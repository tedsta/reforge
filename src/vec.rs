use std::ops::{Add, Sub, Mul};

pub type Vec2f = Vec2<f64>;

#[deriving(Clone, Copy, Encodable, Decodable, Show)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T
}

impl<T: Add<T, T>+Copy> Add<Vec2<T>, Vec2<T>> for Vec2<T> {
    fn add(self, other: Vec2<T>) -> Vec2<T> {
        Vec2{x: self.x + other.x, y: self.y + other.y}
    }
}

impl<T: Sub<T, T>+Copy> Sub<Vec2<T>, Vec2<T>> for Vec2<T> {
    fn sub(self, other: Vec2<T>) -> Vec2<T> {
        Vec2{x: self.x - other.x, y: self.y - other.y}
    }
}

impl<T: Mul<T, T>+Copy> Mul<T, Vec2<T>> for Vec2<T> {
    fn mul(self, other: T) -> Vec2<T> {
        Vec2{x: self.x * other, y: self.y * other}
    }
}

impl<T: Div<T, T>+Copy> Div<T, Vec2<T>> for Vec2<T> {
    fn div(self, other: T) -> Vec2<T> {
        Vec2{x: self.x / other, y: self.y / other}
    }
}