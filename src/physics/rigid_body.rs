use core::f64;
use std::vec;
use graphics::math::Matrix2d;
use graphics::{color, ellipse, line, triangulation, Context, ImageSize, Rectangle, Transformed};
use opengl_graphics::{GlGraphics, Texture};
use piston_window::{texture, TextureSettings};

use crate::Vector2f;
use crate::game::PhysicsSettings;
use crate::physics::shape_type::ShapeType;
use crate::physics::shape::Shape;
use crate::physics::polygon::Polygon;
use crate::physics::circle::Circle;
use super::material::{self, *};
use super::tiled_mesh::TiledMesh;
use super::collision::{self, *};

pub const GRAVITY: Vector2f<f64> = Vector2f { x: 0.0, y: 90.816 };
pub const AIR_RESISTANCE: f64 = 0.08;
pub const BASE_STATIC_FRICTION: f64 = 0.6;
pub const BASE_DYNAMIC_FRICTION: f64 = 0.4;
pub const BASE_ELASTICITY: f64 = 0.5;


#[derive(Clone)]
pub struct RigidBody {
    pub linear_velocity: Vector2f<f64>,
    pub angular_velocity: f64,
    pub material: Material,
    pub is_static: bool,
    pub shape: ShapeType,
    pub mesh: TiledMesh,
}

impl From<Circle> for RigidBody {
    fn from(value: Circle) -> Self {
        Self { 
            linear_velocity: Vector2f::zero(), 
            angular_velocity: 0.0, 
            material: WOOD,
            is_static: false, 
            mesh: TiledMesh::from(&value),
            shape: ShapeType::Circle(value),
        }
    }
}

impl From<Polygon> for RigidBody {
    fn from(value: Polygon) -> Self {
        Self { 
            linear_velocity: Vector2f::zero(), 
            angular_velocity: 0.0, 
            material: WOOD,
            is_static: false, 
            mesh: TiledMesh::from(&value),
            shape: ShapeType::Polygon(value),
        }
    }
}

impl From<ShapeType> for RigidBody {
    fn from(value: ShapeType) -> Self {
        match value {
            ShapeType::Circle(c) => RigidBody::from(c),
            ShapeType::Polygon(p) => RigidBody::from(p),
        }
    }
}

impl RigidBody {
    pub fn new(shape: ShapeType, material: Material, is_static: bool) -> Self {
        Self { 
            linear_velocity: Vector2f::zero(), 
            angular_velocity: 0.0, 
            material,
            is_static, 
            mesh: TiledMesh::from(&shape),
            shape,
        }
    }

    pub fn get_inv_mass(&self) -> f64 {
        if self.is_static { 0.0 } else { 1.0 / (self.shape.area() * self.material.density) }
    }

    pub fn get_inv_inertia(&self) -> f64 {
        if self.is_static { 0.0 } else { 1.0 / (self.shape.momemnt_of_inertia() * self.material.density) }
    }

    pub fn update_velocity(&mut self, dt: f64, physics: &PhysicsSettings) {
        if self.is_static {
            return;
        }

        self.linear_velocity += physics.gravity * dt;        
        
        self.linear_velocity *= 1.0 - physics.air_density * dt;
        self.angular_velocity *= 1.0 - physics.air_density * dt;       
    }

    pub fn update_position(&mut self, dt: f64) {
        self.shape.translate(self.linear_velocity * dt);  
        self.shape.rotate(self.angular_velocity * dt); 
    }

    pub fn collide_with(&mut self, other: &mut RigidBody) -> Option<CollisionData> {
        let push_out = 
        |data: CollisionData, a: &mut RigidBody, b: &mut RigidBody| -> Option<CollisionData> {
            let sep = data.sep_or_t - f64::EPSILON;
            let normal = data.normal;
            match (a.is_static, b.is_static) {
                (true, true) | (false, false) => {
                    a.shape.translate(normal * sep / 2.0);
                    b.shape.translate(normal * -sep / 2.0);
                    
                }
                (true, false) => b.shape.translate(normal * -sep),
                (false, true) => a.shape.translate(normal * sep),
            }
            Some(data)
        };

        let push_circle_out = |p: &mut Polygon, c: &mut Circle| {
            if p.contains_point(c.center) {
                c.center = p.find_closest_surface_point(c.center).0;
            }
        };

        let mut collision_data = None;
        match (&mut self.shape, &mut other.shape) {
            (ShapeType::Circle(a), ShapeType::Circle(b)) => {
                if let Some(collision) = circle_vs_circle(&a, &b) {
                    collision_data = push_out(collision, self, other);
                } 
            },
            (ShapeType::Circle(c), ShapeType::Polygon(p)) => {
                push_circle_out(p, c);
                if let Some(mut collision) = polygon_vs_circle(&p, &c) {
                    collision.normal = -collision.normal;
                    collision_data = push_out(collision, self, other);
                }
            }
            (ShapeType::Polygon(p), ShapeType::Circle(c)) => {
                push_circle_out(p, c);
                if let Some(collision) = polygon_vs_circle(&p, &c) {
                    collision_data = push_out(collision, self, other);
                }
            }
            (ShapeType::Polygon(a), ShapeType::Polygon(b)) => {
                if let Some(collision) = polygon_vs_polygon(&a, &b) {
                    collision_data = push_out(collision, self, other);
                }
            }   
        }

        // Find the contact points
        if let Some(mut collision) = collision_data {
            collision.contacts = match (&self.shape, &other.shape) {
                (ShapeType::Circle(a), ShapeType::Circle(_)) => vec![a.center + collision.normal * a.radius],
                (ShapeType::Circle(a), ShapeType::Polygon(b)) => contact_poly_circle(b, a),
                (ShapeType::Polygon(a), ShapeType::Circle(b)) => contact_poly_circle(a, b),
                (ShapeType::Polygon(a), ShapeType::Polygon(b)) => contact_poly_poly(a, b),
            };
            return Some(collision)
        }

        None
    }


    pub fn resolve_collision(&mut self, other: &mut RigidBody, collision: &CollisionData) {
        let a = self;
        let b = other;

        // Calculate constants
        let a_inv_mass = a.get_inv_mass();
        let a_inv_inertia = a.get_inv_inertia();
        let b_inv_mass = b.get_inv_mass();
        let b_inv_inertia = b.get_inv_inertia();

        let restitution = a.material.restitution.min(b.material.restitution);

        let sf = (a.material.static_friction + b.material.static_friction) / 2.0;
        let df = (a.material.dynamic_friction + b.material.dynamic_friction) / 2.0;

        let normal = collision.normal;
        for contact_point in collision.contacts.as_slice() {
            let ra = *contact_point - a.shape.get_center();
            let rb = *contact_point - b.shape.get_center();
            let a_contact_vel = a.linear_velocity + ra.perpendicular() * a.angular_velocity;
            let b_contact_vel = b.linear_velocity + rb.perpendicular() * b.angular_velocity;
            let relative_velocity = a_contact_vel - b_contact_vel;

            if relative_velocity.dot(normal) < 0.0 { 
                continue; 
            }

            let v_rel = -(1.0 + restitution) * relative_velocity.dot(normal);
            let mut denom = a_inv_mass + b_inv_mass + 
                f64::powi(ra.cross(normal), 2) * a_inv_inertia + 
                f64::powi(rb.cross(normal), 2) * b_inv_inertia;
            denom = denom.max(f64::EPSILON);

            let j = v_rel / denom;
            let a_impulse = normal * j;
            let b_impulse = normal * -j;

            a.linear_velocity += a_impulse * a_inv_mass;
            b.linear_velocity += b_impulse * b_inv_mass;
            a.angular_velocity += ra.cross(a_impulse) * a_inv_inertia;
            b.angular_velocity += rb.cross(b_impulse) * b_inv_inertia;

            // Calculate friction
            let a_contact_vel = a.linear_velocity + ra.perpendicular() * a.angular_velocity;
            let b_contact_vel = b.linear_velocity + rb.perpendicular() * b.angular_velocity;
            let relative_velocity = a_contact_vel - b_contact_vel;
            
            let mut tangent = relative_velocity - normal * relative_velocity.dot(normal);
            if tangent.nearly_equal(Vector2f::zero(), 0.0005) {
                continue;
            }

            tangent = tangent.normalize();

            let v_rel = -relative_velocity.dot(tangent);
            let mut denom = a_inv_mass + b_inv_mass +
                f64::powi(ra.cross(tangent), 2) * a_inv_inertia + 
                f64::powi(rb.cross(tangent), 2) * b_inv_inertia;
            denom = denom.max(f64::EPSILON);

            let mut jt = v_rel / denom;
            if jt.abs() > -j * sf {
                jt = j * df;
            }
            
            let a_friction_impulse = tangent * jt;
            let b_friction_impulse = tangent * -jt;

            a.linear_velocity += a_friction_impulse * a_inv_mass;
            b.linear_velocity += b_friction_impulse * b_inv_mass;
            a.angular_velocity += ra.cross(a_friction_impulse) * a_inv_inertia;
            b.angular_velocity += rb.cross(b_friction_impulse) * b_inv_inertia;
        }
    }

    pub fn scale(&self, ratio: f64) -> Self {
        Self::new(self.shape.scale(ratio), self.material, self.is_static)
    }

    pub fn draw(&self, transform: Matrix2d, texture: &Texture, c: Context, gl: &mut GlGraphics) {
        self.mesh.draw(
            transform.trans_pos(self.shape.get_center()).rot_rad(self.shape.get_rotation()), 
            color::WHITE, texture, c, gl);
    }
}