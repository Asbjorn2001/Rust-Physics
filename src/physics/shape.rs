use graphics::math::Matrix2d;

use crate::Vector2f;
use crate::GlGraphics;

pub trait Renderable {
    fn draw(&self, tansform: Matrix2d, gl: &mut GlGraphics);
}

pub trait Shape : Renderable {
    fn area(&self) -> f64;

    fn momemnt_of_inertia(&self) -> f64;

    fn contains_point(&self, point: Vector2f<f64>) -> bool;
}