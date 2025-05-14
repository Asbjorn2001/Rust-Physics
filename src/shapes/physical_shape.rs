use graphics::color;
use rand::seq;

use crate::shapes::physics::*;
use crate::Vector2f;
use crate::Renderable;
use crate::shapes::circle::Circle;
use crate::shapes::polygon::Polygon;
use crate::clone::Clone;
use crate::Context;
use crate::GlGraphics;
use crate::shapes::geometry::Geometry;

#[derive(Clone)]
pub enum PhysicalShape {
    Circle(Physical<Circle>),
    Polygon(Physical<Polygon>)
}

impl From<Physical<Circle>> for PhysicalShape {
    fn from(value: Physical<Circle>) -> Self {
        Self::Circle(value)
    }
}

impl Renderable for PhysicalShape {
    fn draw(&self, c: Context, gl: &mut GlGraphics) {
        match self {
            PhysicalShape::Circle(circle) => circle.shape.draw(c, gl),
            PhysicalShape::Polygon(poly) => poly.shape.draw(c, gl),
        }
    }
}

impl Geometry for PhysicalShape {
    fn area(&self) -> f64 {
        match self {
            PhysicalShape::Circle(c) => c.shape.area(),
            PhysicalShape::Polygon(p) => p.shape.area(),
        }
    }   

    fn contains_point(&self, point: Vector2f<f64>) -> bool {
        match self {
            PhysicalShape::Circle(c) => c.shape.contains_point(point),
            PhysicalShape::Polygon(p) => p.shape.contains_point(point)
        }
    }
}

impl Physics for PhysicalShape {
    fn momemnt_of_inertia(&self) -> f64 {
        match self {
            PhysicalShape::Circle(c) => c.momemnt_of_inertia(),
            PhysicalShape::Polygon(p) => p.momemnt_of_inertia(),   
        }
    }

    fn update_vectors(&mut self, dt: f64) {
        match self {
            PhysicalShape::Circle(c) => c.update_vectors(dt),
            PhysicalShape::Polygon(p) => p.update_vectors(dt),
        }
    }

    fn resolve_collision_with(&mut self, other: &mut PhysicalShape) -> bool {
        match self {
            PhysicalShape::Circle(c) => c.resolve_collision_with(other),
            PhysicalShape::Polygon(p) => p.resolve_collision_with(other),
        }
    }

    fn resolve_border_collision(&mut self, dimensions: Vector2f<f64>) -> bool {
        match self {
            PhysicalShape::Circle(c) => c.resolve_border_collision(dimensions),
            PhysicalShape::Polygon(p) => p.resolve_border_collision(dimensions),
        }
    }
}

impl PhysicalShape {
    pub fn set_velocity(&mut self, velocity: Vector2f<f64>) {
        match self {
            PhysicalShape::Circle(c) => c.velocity = velocity,
            PhysicalShape::Polygon(p) => p.velocity = velocity,
        }
    }

    pub fn change_color(&mut self, color: [f32; 4]) {
        match self {
            PhysicalShape::Circle(c) => c.shape.color = color,
            PhysicalShape::Polygon(p) => p.shape.color = color,
        }
    }

    pub fn center_around(&mut self, position: Vector2f<f64>) {
        match self {
            PhysicalShape::Circle(c) => c.shape.center = position,
            PhysicalShape::Polygon(p) => p.shape.center = position,
        }
    }
}