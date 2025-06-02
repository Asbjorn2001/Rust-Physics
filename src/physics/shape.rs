use graphics::math::Matrix2d;

use crate::Vector2f;
use crate::GlGraphics;

use super::collision::AABB;

pub trait Renderable {
    fn draw(&self, transform: Matrix2d, gl: &mut GlGraphics, color: [f32; 4]);
}

pub trait Shape : Renderable {
    fn area(&self) -> f64;

    fn momemnt_of_inertia(&self) -> f64;

    fn get_aabb(&self) -> AABB;

    fn contains_point(&self, point: Vector2f<f64>) -> bool;

    // Returns closest surface point and surface normal
    fn find_closest_surface_point(&self, point: Vector2f<f64>) -> (Vector2f<f64>, Vector2f<f64>);
}