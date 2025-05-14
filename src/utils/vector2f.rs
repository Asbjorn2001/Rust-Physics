use num_traits::*;
use core::str;
use std::ops::*;
use rand::Rng;


#[derive(Clone, Copy, Debug)]
pub struct Vector2f<T: Float> {
    pub x: T,
    pub y: T,
}

#[allow(dead_code)]
impl<T: Float> Vector2f<T> {
    pub fn new(x: T, y: T) -> Self {
        Vector2f { x: x, y: y }
    }

    pub fn dot(self, rhs: Vector2f<T>) -> T {
        let v = self * rhs;
        v.x + v.y
    }

    pub fn cross(self, rhs: Vector2f<T>) -> T {
        self.x * rhs.y - self.y * rhs.x
    }

    pub fn len(&self) -> T {
        T::sqrt(T::powi(self.x, 2) + T::powi(self.y, 2))
    }

    pub fn normalize(&self) -> Self {
        let len = self.len();
        Vector2f { x: self.x / len, y: self.y / len }
    }

    pub fn perpendicular(&self) -> Self {
        Vector2f { x: -self.y, y: self.x }
    }

    pub fn rotate(&self, radians: T) -> Self {
        let cos = T::cos(radians);
        let sin = T::sin(radians);
        Vector2f { x: self.x * cos - self.y * sin, 
                 y: self.x * sin + self.y * cos }
    }
}

impl Vector2f<f64> {
    pub fn random_direction() -> Self {
        let mut r = rand::rng();
        let x = r.random_range(-1.0..1.0);
        let y = r.random_range(-1.0..1.0);
        Vector2f { x, y }.normalize()
    }

    pub fn zero() -> Self {
        Vector2f { x: 0.0, y: 0.0 }
    }
}

impl<T: Float> From<[T; 2]> for Vector2f<T> where T: Copy {
    fn from(value: [T; 2]) -> Self {
        Vector2f { x: value[0], y: value[1] }
    }
}

impl <T: Float> From<Vector2f<T>> for [T; 2] {
    fn from(value: Vector2f<T>) -> Self {
        [value.x, value.y]
    }
}

impl<T: Float> Add<Vector2f<T>> for Vector2f<T> {
    type Output = Vector2f<T>;

    fn add(self, rhs: Vector2f<T>) -> Self::Output {
        Vector2f { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}

impl<T: Float> AddAssign for Vector2f<T> where T: AddAssign {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl<T: Float> Sub<Vector2f<T>> for Vector2f<T> {
    type Output = Vector2f<T>;

    fn sub(self, rhs: Vector2f<T>) -> Self::Output {
        Vector2f { x: self.x - rhs.x, y: self.y - rhs.y }
    }
}

impl<T: Float> SubAssign for Vector2f<T> where T: SubAssign {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl<T: Float> Neg for Vector2f<T> {
    type Output = Vector2f<T>;
    
    fn neg(self) -> Self::Output {
        Vector2f { x: -self.x, y: -self.y}
    }
}

impl<T: Float> Mul<Vector2f<T>> for Vector2f<T> {
    type Output = Vector2f<T>;

    fn mul(self, rhs: Vector2f<T>) -> Self::Output {
        Vector2f { x: self.x * rhs.x, y: self.y * rhs.y }
    }
}

impl<T: Float> MulAssign for Vector2f<T> where T: MulAssign {
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
    }
}

impl<T: Float> Mul<T> for Vector2f<T> where T: Copy {
    type Output = Vector2f<T>;

    fn mul(self, rhs: T) -> Self::Output {
        Vector2f {x: self.x * rhs, y: self.y * rhs}
    }
}

impl<T: Float> MulAssign<T> for Vector2f<T> where T: MulAssign, T: Copy {
    fn mul_assign(&mut self, rhs: T) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl<T: Float> Div<Vector2f<T>> for Vector2f<T> {
    type Output = Vector2f<T>;

    fn div(self, rhs: Vector2f<T>) -> Self::Output {
        Vector2f { x: self.x / rhs.x, y: self.y / rhs.y }
    }
}

impl<T: Float> Div<T> for Vector2f<T> where T: Copy {
    type Output = Vector2f<T>;

    fn div(self, rhs: T) -> Self::Output {
        Vector2f { x: self.x / rhs, y: self.y / rhs }
    }
}

impl<T: Float> DivAssign<T> for Vector2f<T> where T: DivAssign, T: Copy {
    fn div_assign(&mut self, rhs: T) {
        self.x /= rhs;
        self.y /= rhs;
    }
}