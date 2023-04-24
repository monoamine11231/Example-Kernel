use core::intrinsics::sqrtf64;

#[derive(Copy, Clone, Eq, PartialEq)]
struct Vec2 {
    x: i32,
    y: i32,
}

impl Vec2 {
    pub fn magnitude(&self) -> f64 {
        return ((self.x * self.x + self.y * self.y) as f64).powf(0.5);
    }
}
