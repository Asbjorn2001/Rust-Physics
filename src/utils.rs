use num_traits::*;
use core::str;
use std::ops::*;
use rand::Rng;

pub fn get_pair_mut<T>(vec: &mut Vec<T>, i: usize, j: usize) -> (&mut T, &mut T) {
    assert!(i != j);
    if i < j {
        let (head, tail) = vec.split_at_mut(j);
        (&mut head[i], &mut tail[0])
    } else {
        let (head, tail) = vec.split_at_mut(i);
        (&mut tail[0], &mut head[j])
    }
}

#[derive(Clone, Copy)]
pub struct Delta<T> {
    pub curr: T,
    pub next: T,
}

impl<T> Delta<T> {
    pub fn new(value: T) -> Self where T: Copy {
        Delta { curr: value, next: value }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Vector2<T: Float> {
    pub x: T,
    pub y: T,
}

#[allow(dead_code)]
impl<T: Float> Vector2<T> {
    pub fn new(x: T, y: T) -> Self {
        Vector2 { x: x, y: y }
    }

    pub fn dot(self, rhs: Vector2<T>) -> T {
        let v = self * rhs;
        v.x + v.y
    }

    pub fn cross(self, rhs: Vector2<T>) -> T {
        self.x * rhs.y - self.y * rhs.x
    }

    pub fn len(&self) -> T {
        T::sqrt(T::powi(self.x, 2) + T::powi(self.y, 2))
    }

    pub fn normalize(&self) -> Self {
        let len = self.len();
        Vector2 { x: self.x / len, y: self.y / len }
    }

    pub fn perpendicular(&self) -> Self {
        Vector2 { x: -self.y, y: self.x }
    }

    pub fn rotate(&self, radians: T) -> Self {
        let cos = T::cos(radians);
        let sin = T::sin(radians);
        Vector2 { x: self.x * cos - self.y * sin, 
                 y: self.x * sin + self.y * cos }
    }
}

impl Vector2<f64> {
    pub fn random_direction() -> Self {
        let mut r = rand::rng();
        let x = r.random_range(-1.0..1.0);
        let y = r.random_range(-1.0..1.0);
        Vector2 { x, y }.normalize()
    }

    pub fn zero() -> Self {
        Vector2 { x: 0.0, y: 0.0 }
    }
}

impl<T: Float> From<[T; 2]> for Vector2<T> where T: Copy {
    fn from(value: [T; 2]) -> Self {
        Vector2 { x: value[0], y: value[1] }
    }
}

impl <T: Float> From<Vector2<T>> for [T; 2] {
    fn from(value: Vector2<T>) -> Self {
        [value.x, value.y]
    }
}

impl<T: Float> Add<Vector2<T>> for Vector2<T> {
    type Output = Vector2<T>;

    fn add(self, rhs: Vector2<T>) -> Self::Output {
        Vector2 { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}

impl<T: Float> AddAssign for Vector2<T> where T: AddAssign {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl<T: Float> Sub<Vector2<T>> for Vector2<T> {
    type Output = Vector2<T>;

    fn sub(self, rhs: Vector2<T>) -> Self::Output {
        Vector2 { x: self.x - rhs.x, y: self.y - rhs.y }
    }
}

impl<T: Float> SubAssign for Vector2<T> where T: SubAssign {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl<T: Float> Neg for Vector2<T> {
    type Output = Vector2<T>;
    
    fn neg(self) -> Self::Output {
        Vector2 { x: -self.x, y: -self.y}
    }
}

impl<T: Float> Mul<Vector2<T>> for Vector2<T> {
    type Output = Vector2<T>;

    fn mul(self, rhs: Vector2<T>) -> Self::Output {
        Vector2 { x: self.x * rhs.x, y: self.y * rhs.y }
    }
}

impl<T: Float> MulAssign for Vector2<T> where T: MulAssign {
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
    }
}

impl<T: Float> Mul<T> for Vector2<T> where T: Copy {
    type Output = Vector2<T>;

    fn mul(self, rhs: T) -> Self::Output {
        Vector2 {x: self.x * rhs, y: self.y * rhs}
    }
}

impl<T: Float> MulAssign<T> for Vector2<T> where T: MulAssign, T: Copy {
    fn mul_assign(&mut self, rhs: T) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl<T: Float> Div<Vector2<T>> for Vector2<T> {
    type Output = Vector2<T>;

    fn div(self, rhs: Vector2<T>) -> Self::Output {
        Vector2 { x: self.x / rhs.x, y: self.y / rhs.y }
    }
}

impl<T: Float> Div<T> for Vector2<T> where T: Copy {
    type Output = Vector2<T>;

    fn div(self, rhs: T) -> Self::Output {
        Vector2 { x: self.x / rhs, y: self.y / rhs }
    }
}

impl<T: Float> DivAssign<T> for Vector2<T> where T: DivAssign, T: Copy {
    fn div_assign(&mut self, rhs: T) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

/* 
pub struct Matrix2x2<T: Signed> {
    rows: [Vector<T>; 2],
}

impl<T: Signed + Float> Mul<Vector<T>> for Matrix2x2<T> {
    type Output = Vector<T>;

    fn mul(self, rhs: Vector<T>) -> Self::Output {
        Vector { x: self.rows[0].dot(rhs), y: self.rows[1].dot(rhs) }
    }
}

impl<T: Signed + Float> Matrix2x2<T> {
    pub fn get_rotation_matrix
}
*/