use std::f64::consts::PI;

use crate::Vector2f;
use crate::Renderable;
use crate::Context;
use crate::GlGraphics;
use crate::shapes::geometry::Geometry;

#[derive(Clone, Copy)]
pub struct Circle {
    pub radius: f64,
    pub center: Vector2f<f64>,
    pub color: [f32; 4],
}

impl Renderable for Circle {
    fn draw(&self, c: Context, gl: &mut GlGraphics) {
        let center = self.center;
        let square = graphics::rectangle::centered_square(center.x, center.y, self.radius);
        
        graphics::ellipse(self.color, square, c.transform, gl);
    }
}

impl Geometry for Circle {
    fn area(&self) -> f64 {
        self.radius * self.radius * PI
    }

    fn contains_point(&self, point: Vector2f<f64>) -> bool {
        (self.center - point).len() < self.radius
    }
}

impl Circle {
    pub fn new(center: Vector2f<f64>, radius: f64, color: [f32; 4]) -> Self {
        Self { 
            radius: radius, 
            center: center, 
            color: color, 
        }
    }
}