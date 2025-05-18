use crate::Vector2f;
use crate::Context;
use crate::GlGraphics;

pub trait Renderable {
    fn draw(&self, c: Context, gl: &mut GlGraphics);
}

pub trait Shape : Renderable {
    fn area(&self) -> f64;

    fn momemnt_of_inertia(&self) -> f64;

    fn contains_point(&self, point: Vector2f<f64>) -> bool;
}