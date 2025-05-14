use crate::Vector2f;

pub trait Geometry {
    fn area(&self) -> f64;

    fn contains_point(&self, point: Vector2f<f64>) -> bool;
}