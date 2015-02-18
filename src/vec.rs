use std::ops::{Add, Sub, Mul, Div};

pub type Vec2f = Vec2<f64>;

#[derive(Clone, Copy, RustcEncodable, RustcDecodable, Debug)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

impl<T: Add+Copy> Add for Vec2<T> {
    type Output = Vec2<<T as Add>::Output>;

    fn add(self, other: Vec2<T>) -> Vec2<<T as Add>::Output> {
        Vec2{x: self.x + other.x, y: self.y + other.y}
    }
}

impl<T: Sub+Copy> Sub for Vec2<T> {
    type Output = Vec2<<T as Sub>::Output>;

    fn sub(self, other: Vec2<T>) -> Vec2<<T as Sub>::Output> {
        Vec2{x: self.x - other.x, y: self.y - other.y}
    }
}

impl<T: Mul+Copy> Mul<T> for Vec2<T> {
    type Output = Vec2<<T as Mul>::Output>;

    fn mul(self, other: T) -> Vec2<<T as Mul>::Output> {
        Vec2{x: self.x * other, y: self.y * other}
    }
}

impl<T: Div+Copy> Div<T> for Vec2<T> {
    type Output = Vec2<<T as Div>::Output>;

    fn div(self, other: T) -> Vec2<<T as Div>::Output> {
        Vec2{x: self.x / other, y: self.y / other}
    }
}
