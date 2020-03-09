use core::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub};

#[derive(Copy, Clone)]
pub struct V2 {
    pub x: f32,
    pub y: f32,
}

impl V2 {
    pub fn new(x: f32, y: f32) -> V2 {
        V2 { x, y }
    }

    pub fn zero() -> V2 {
        V2 { x: 0.0, y: 0.0 }
    }
}

impl Add<V2> for V2 {
    type Output = V2;

    fn add(self, rhs: V2) -> Self::Output {
        V2::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl AddAssign<V2> for V2 {
    fn add_assign(&mut self, rhs: V2) {
        *self = *self + rhs;
    }
}

impl AddAssign<f32> for V2 {
    fn add_assign(&mut self, rhs: f32) {
        self.x += rhs;
        self.y += rhs;
    }
}

impl Sub<V2> for V2 {
    type Output = V2;

    fn sub(self, rhs: V2) -> Self::Output {
        V2::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Neg for V2 {
    type Output = V2;

    fn neg(self) -> Self::Output {
        V2::new(-self.x, -self.y)
    }
}

impl Mul<f32> for V2 {
    type Output = V2;

    fn mul(self, rhs: f32) -> Self::Output {
        V2::new(self.x * rhs, self.y * rhs)
    }
}

impl Mul<V2> for f32 {
    type Output = V2;

    fn mul(self, rhs: V2) -> Self::Output {
        rhs * self
    }
}

impl MulAssign<f32> for V2 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl Mul<V2> for V2 {
    type Output = f32;

    fn mul(self, rhs: V2) -> Self::Output {
        self.x * rhs.x + self.y * rhs.y
    }
}
