use core::f64;
use std::f64::consts::PI;
use std::vec;

use crate::Vector2f;
use crate::shapes::physical_shape::PhysicalShape;
use crate::shapes::geometry::Geometry;
use crate::shapes::collision::*;
use crate::shapes::polygon::Polygon;
use crate::shapes::circle::Circle;

const GRAVITY: Vector2f<f64> = Vector2f { x: 0.0, y: 90.816 };
const AIR_RESISTANCE: f64 = 0.08;
const BASE_FRICTION: f64 = 0.5;

pub trait Physics {
    fn momemnt_of_inertia(&self) -> f64;

    fn update_vectors(&mut self, dt: f64);

    fn resolve_collision_with(&mut self, other: &mut PhysicalShape) -> bool;
    
    fn resolve_border_collision(&mut self, dimensions: Vector2f<f64>) -> bool;
}

#[derive(Clone)]
pub struct Physical<T: Geometry> {
    pub velocity: Vector2f<f64>,
    pub angular_velocity: f64,
    pub elasticity: f64,
    pub friction: f64,
    pub immovable: bool,
    pub shape: T,
}

impl From<Circle> for Physical<Circle> {
    fn from(value: Circle) -> Self {
        Self { 
            velocity: Vector2f::zero(), 
            angular_velocity: 0.0, 
            elasticity: 0.5, 
            friction: 0.0,
            immovable: false, 
            shape: value
        }
    }
}

impl From<Polygon> for Physical<Polygon> {
    fn from(value: Polygon) -> Self {
        Self { 
            velocity: Vector2f::zero(), 
            angular_velocity: 0.0, 
            elasticity: 0.5, 
            friction: 0.0,
            immovable: false, 
            shape: value
        }
    }
}

impl Physics for Physical<Circle> {
    fn momemnt_of_inertia(&self) -> f64 {
        PI * f64::powi(self.shape.radius, 4) / 4.0
    }

    fn update_vectors(&mut self, dt: f64) {
        self.shape.center += self.velocity * dt;
        self.velocity += GRAVITY * dt;        
        self.velocity *= 1.0 - (AIR_RESISTANCE + self.friction) * dt;

        self.friction = 0.0
    }

    fn resolve_collision_with(&mut self, other: &mut PhysicalShape) -> bool {
        match other {
            PhysicalShape::Circle(c) => resolve_collision_circle_circle(self, c),
            PhysicalShape::Polygon(p) => resolve_collision_poly_circle(p, self),
        }
    }

    fn resolve_border_collision(&mut self, dimensions: Vector2f<f64>) -> bool {        
        let center = self.shape.center;
        let velocity = self.velocity;
        let radius = self.shape.radius;
        let mut collision = false;
        if center.x <= radius {
            self.shape.center.x = radius;
            self.velocity.x = -velocity.x * self.elasticity;
            collision = true;
        } else if center.x >= (dimensions.x - radius) {
            self.shape.center.x = dimensions.x - radius;
            self.velocity.x = -velocity.x * self.elasticity;
            collision = true;
        }

        if center.y <= radius {
            self.shape.center.y = radius;
            self.velocity.y = -velocity.y * self.elasticity;
            collision = true;
        }
        else if center.y >= (dimensions.y - radius) {
            self.shape.center.y = dimensions.y - radius;
            self.velocity.y = -velocity.y * self.elasticity;
            collision = true;
        }

        if collision { self.friction = BASE_FRICTION };

        collision
    }
}

impl Physics for Physical<Polygon> {
    fn momemnt_of_inertia(&self) -> f64 {
        let n = self.shape.local_vertices.len();
        let mut I = 0.0; 

        for i in 0..n {
            let p1 = self.shape.local_vertices[i];
            let p2 = self.shape.local_vertices[(i + 1) % n];
            I += p1.cross(p2) * (p1.dot(p1) + p1.dot(p2) + p2.dot(p2));
        }
        
        (I / 12.0).abs()
    }

    fn update_vectors(&mut self, dt: f64) {
        self.shape.center += self.velocity * dt;
        self.velocity += GRAVITY * dt;
        self.velocity *= 1.0 - (AIR_RESISTANCE + self.friction) * dt;

        self.shape.rotation += self.angular_velocity * dt;
        self.angular_velocity *= 1.0 - (AIR_RESISTANCE + self.friction) * dt;

        self.friction = 0.0;
    }

    fn resolve_collision_with(&mut self, other: &mut PhysicalShape) -> bool {
        match other {
            PhysicalShape::Circle(c) => resolve_collision_poly_circle(self, c),
            PhysicalShape::Polygon(p) => resolve_collision_poly_poly(self, p),
        }
    }

    fn resolve_border_collision(&mut self, dimensions: Vector2f<f64>) -> bool {
        let mut seperation = f64::INFINITY;
        let mut normal = Vector2f::zero();
        let mut pen_verts = vec![];

        let mut push_vert = |sep: f64, norm: Vector2f<f64>, vert: Vector2f<f64>| {
            pen_verts.push(vert);
            if sep < seperation {
                seperation = sep;
                normal = norm;
            }
        };
        
        for v in self.shape.get_vertices() {
            if v.x < 0.0 {
                push_vert(v.x, Vector2f::new(1.0, 0.0), v);                
            } else if v.x > dimensions.x {
                push_vert(dimensions.x - v.x, Vector2f::new(-1.0, 0.0), v);                                
            }

            if v.y < 0.0 {
                push_vert(v.y, Vector2f::new(0.0, 1.0), v);                
            } else if v.y > dimensions.y {
                push_vert(dimensions.y - v.y, Vector2f::new(0.0, -1.0), v);                
            }
        }

        if seperation < 0.0 {
            let mut contact_point = Vector2f::zero();
            pen_verts.iter().for_each(|v| contact_point += *v); 
            contact_point /= pen_verts.len() as f64;

            let r = contact_point - self.shape.center;
            self.shape.center -= normal * seperation;
            
            let contact_velocity = self.velocity + r.perpendicular() * self.angular_velocity;
            let v_rel = contact_velocity.dot(normal);

            let denom = 1.0 / self.shape.area() + f64::powi(r.cross(normal), 2) / self.momemnt_of_inertia();
            let j = -(1.0 + self.elasticity) * v_rel / denom;
            
            let impulse = normal * j;
            self.velocity += impulse / self.shape.area();
            self.angular_velocity += r.cross(impulse) / self.momemnt_of_inertia();

            self.friction = BASE_FRICTION;

            return true;
        }

        false
    }
}