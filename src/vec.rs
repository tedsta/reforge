use num::Float;
use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Add, Sub, Mul, Div};
use std::str::FromStr;

pub type Vec2f = Vec2<f64>;

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

impl<T> Vec2<T> {
    pub fn new(x: T, y: T) -> Vec2<T> {
        Vec2 { x: x, y: y }
    }
}

impl<T: Float> Vec2<T> {
    pub fn normalize(self) -> Vec2<T> {
        Vec2 { x: self.x / self.length(), y: self.y / self.length() }
    }
    
    pub fn dot(self, other: Vec2<T>) -> T {
        self.x*other.x + self.y*other.y
    }
    
    pub fn length(&self) -> T {
        (self.x*self.x + self.y*self.y).sqrt()
    }
    
    pub fn rotate(self, radians: T) -> Vec2<T> {
        let px = self.x * radians.cos() - self.y * radians.sin(); 
        let py = self.x * radians.sin() + self.y * radians.cos();
        Vec2 { x: px, y: py }
    }
    
    pub fn floor(self) -> Vec2<T> {
        Vec2 { x: self.x.floor(), y: self.y.floor() }
    }
    
    pub fn ceil(self) -> Vec2<T> {
        Vec2 { x: self.x.ceil(), y: self.y.ceil() }
    }
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

impl<T: FromStr+Copy> FromStr for Vec2<T> {
    type Err = ParseVecError;

    fn from_str(s: &str) -> Result<Vec2<T>, ParseVecError> {
        if s.len() > 0 && s.as_bytes()[0] == b'(' && s.as_bytes()[s.len()-1] == b')' {
            let components: Vec<T> =
                s[1..s.len()-1].split(',')
                               .map(|s| { s.trim_left().trim_right() })
                               .map(|s| { s.parse().ok().unwrap() })
                               .collect();
            Ok(Vec2::new(components[0], components[1]))
        } else {
            Err(ParseVecError)
        }
    }
}

pub struct ParseVecError;

impl Error for ParseVecError {
    fn description(&self) -> &str {
        "Couldn't parse the vector"
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}

impl Debug for ParseVecError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        Ok(try!(f.write_str("Make sure your vector is all like this: (x, y)")))
    }
}

impl Display for ParseVecError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        Ok(try!(f.write_str("Make sure your vector is all like this: (x, y)")))
    }
}
