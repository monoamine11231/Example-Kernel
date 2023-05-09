use core::ops;
use core::ops::{Add, Mul, Sub};

use crate::math::utils::*;

#[derive(Copy, Clone)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

impl Vec2<f32> {
    pub fn new(_x: f32, _y: f32) -> Self {
        return Vec2 { x: _x, y: _y };
    }
}
impl Vec2<usize> {
    pub fn new(_x: usize, _y: usize) -> Self {
        return Vec2 { x: _x, y: _y };
    }
}

//Vector magnitude
impl Vec2<f32> {
    pub fn magnitude(&self) -> f32 {
        return sqrt(((self.x * self.x) + (self.y * self.y)) as f32);
    }
}
impl Vec2<usize> {
    pub fn magnitude(&self) -> f32 {
        return sqrt(((self.x * self.x) + (self.y * self.y)) as f32);
    }
}

//Vector add
impl ops::Add<Vec2<f32>> for Vec2<f32> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        return Vec2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        };
    }
}
impl ops::Add<Vec2<usize>> for Vec2<usize> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        return Vec2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        };
    }
}

//Vector add_assign
impl ops::AddAssign<Vec2<f32>> for Vec2<f32> {
    fn add_assign(&mut self, rhs: Vec2<f32>) {
        self.x = self.x + rhs.x;
        self.y = self.y + rhs.y;
    }
}
impl ops::AddAssign<Vec2<usize>> for Vec2<usize> {
    fn add_assign(&mut self, rhs: Vec2<usize>) {
        self.x = self.x + rhs.x;
        self.y = self.y + rhs.y;
    }
}

//Vector sub_assign
impl ops::SubAssign<Vec2<f32>> for Vec2<f32> {
    fn sub_assign(&mut self, rhs: Vec2<f32>) {
        self.x = self.x - rhs.x;
        self.y = self.y - rhs.y;
    }
}
impl ops::SubAssign<Vec2<usize>> for Vec2<usize> {
    fn sub_assign(&mut self, rhs: Vec2<usize>) {
        self.x = self.x - rhs.x;
        self.y = self.y - rhs.y;
    }
}

//Vector subtract
impl ops::Sub<Vec2<f32>> for Vec2<f32> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        return Vec2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        };
    }
}
impl ops::Sub<Vec2<usize>> for Vec2<usize> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        return Vec2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        };
    }
}

//Vector dot product
impl ops::Mul<Vec2<f32>> for Vec2<f32> {
    type Output = f32;
    fn mul(self, rhs: Self) -> Self::Output {
        return self.x * rhs.x + self.y * rhs.y;
    }
}
impl ops::Mul<Vec2<usize>> for Vec2<usize> {
    type Output = usize;
    fn mul(self, rhs: Self) -> Self::Output {
        return self.x * rhs.x + self.y * rhs.y;
    }
}

//Vector multiply with scalar
impl ops::Mul<f32> for Vec2<f32> {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
        return Vec2 {
            x: self.x * rhs,
            y: self.y * rhs,
        };
    }
}
impl ops::Mul<usize> for Vec2<usize> {
    type Output = Self;
    fn mul(self, rhs: usize) -> Self::Output {
        return Vec2 {
            x: self.x * rhs,
            y: self.y * rhs,
        };
    }
}
