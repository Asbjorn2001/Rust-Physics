use crate::Vector2f;
use crate::physics::shape::Renderable;
use crate::physics::circle::Circle;
use crate::physics::polygon::Polygon;
use crate::clone::Clone;
use crate::Context;
use crate::GlGraphics;
use crate::physics::shape::Shape;

#[derive(Clone)]
pub enum ShapeType {
    Circle(Circle),
    Polygon(Polygon),
}

impl Renderable for ShapeType {
    fn draw(&self, c: Context, gl: &mut GlGraphics) {
        match self {
            ShapeType::Circle(circle) => circle.draw(c, gl),
            ShapeType::Polygon(poly) => poly.draw(c, gl),
        }
    }
}

impl Shape for ShapeType {
    fn area(&self) -> f64 {
        match self {
            ShapeType::Circle(c) => c.area(),
            ShapeType::Polygon(p) => p.area(),
        }
    }   

    fn momemnt_of_inertia(&self) -> f64 {
        match self {
            ShapeType::Circle(c) => c.momemnt_of_inertia(),
            ShapeType::Polygon(p) => p.momemnt_of_inertia(),
        }
    }

    fn contains_point(&self, point: Vector2f<f64>) -> bool {
        match self {
            ShapeType::Circle(c) => c.contains_point(point),
            ShapeType::Polygon(p) => p.contains_point(point)
        }
    }
}

impl ShapeType {
    pub fn get_center(&self) -> Vector2f<f64> {
        match self {
            ShapeType::Circle(c) => c.center,
            ShapeType::Polygon(p) => p.center,
        }
    }
    
    pub fn set_center(&mut self, position: Vector2f<f64>) {
        match self {
            ShapeType::Circle(c) => c.center = position,
            ShapeType::Polygon(p) => p.center = position,
        }
    }

    pub fn translate(&mut self, translation: Vector2f<f64>) {
        match self {
            ShapeType::Circle(c) => c.center += translation,
            ShapeType::Polygon(p) => p.center += translation,
        }
    }

    pub fn get_rotation(&self) -> f64 {
        match self {
            ShapeType::Circle(c) => c.rotation,
            ShapeType::Polygon(p) => p.rotation,
        }
    } 

    pub fn set_rotation(&mut self, rotation: f64) {
        match self {
            ShapeType::Circle(c) => c.rotation = rotation,
            ShapeType::Polygon(p) => p.rotation = rotation,
        }
    }

    pub fn rotate(&mut self, radians: f64) {
        match self {
            ShapeType::Circle(c) => c.rotation += radians,
            ShapeType::Polygon(p) => p.rotation += radians,
        }
    }

    pub fn get_color(&self) -> [f32; 4] {
        match self {
            ShapeType::Circle(c) => c.color,
            ShapeType::Polygon(p) => p.color,
        }
    }

    pub fn set_color(&mut self, color: [f32; 4]) {
        match self {
            ShapeType::Circle(c) => c.color = color,
            ShapeType::Polygon(p) => p.color = color,
        }
    }
}