use core::f64;

use crate::Vector2f;
use crate::game::PhysicsSettings;
use crate::physics::shape_type::ShapeType;
use crate::physics::shape::Shape;
use crate::physics::polygon::Polygon;
use crate::physics::circle::Circle;

use super::collision::*;

pub const GRAVITY: Vector2f<f64> = Vector2f { x: 0.0, y: 90.816 };
pub const AIR_RESISTANCE: f64 = 0.08;
pub const BASE_STATIC_FRICTION: f64 = 0.6;
pub const BASE_DYNAMIC_FRICTION: f64 = 0.4;
pub const BASE_ELASTICITY: f64 = 0.5;


#[derive(Clone)]
pub struct RigidBody {
    pub linear_velocity: Vector2f<f64>,
    pub angular_velocity: f64,
    pub elasticity: f64,
    pub static_friction: f64,
    pub dynamic_friction: f64,
    pub is_static: bool,
    pub shape: ShapeType,
}

impl From<Circle> for RigidBody {
    fn from(value: Circle) -> Self {
        Self { 
            linear_velocity: Vector2f::zero(), 
            angular_velocity: 0.0, 
            elasticity: BASE_ELASTICITY, 
            static_friction: BASE_STATIC_FRICTION,
            dynamic_friction: BASE_DYNAMIC_FRICTION,
            is_static: false, 
            shape: ShapeType::Circle(value),
        }
    }
}

impl From<Polygon> for RigidBody {
    fn from(value: Polygon) -> Self {
        Self { 
            linear_velocity: Vector2f::zero(), 
            angular_velocity: 0.0, 
            elasticity: BASE_ELASTICITY, 
            static_friction: BASE_STATIC_FRICTION,
            dynamic_friction: BASE_DYNAMIC_FRICTION,
            is_static: false, 
            shape: ShapeType::Polygon(value),
        }
    }
}

impl RigidBody {
    pub fn new(shape: ShapeType, elasticity: f64, static_friction: f64, dynamic_friction: f64, is_static: bool) -> Self {
        Self { 
            linear_velocity: Vector2f::zero(), 
            angular_velocity: 0.0, 
            elasticity, 
            static_friction, 
            dynamic_friction, 
            is_static, 
            shape 
        }
    }

    pub fn update_vectors(&mut self, dt: f64, physics: &PhysicsSettings) {
        if self.is_static {
            return;
        }

        self.linear_velocity += physics.gravity * dt;
        self.linear_velocity *= 1.0 - physics.air_density * dt;

        self.angular_velocity *= 1.0 - physics.air_density * dt;

        self.shape.translate(self.linear_velocity * dt);  
        self.shape.rotate(self.angular_velocity * dt);
    }

    pub fn collide_with(&mut self, other: &mut RigidBody) -> Option<CollisionData> {
        let push_out = 
        |data: CollisionData, a: &mut RigidBody, b: &mut RigidBody| -> Option<CollisionData> {
            let CollisionData(sep, normal) = data;
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

        match (&self.shape, &other.shape) {
            (ShapeType::Circle(a), ShapeType::Circle(b)) => {
                if let Some(collision) = collision_circle_circle(a, b) {
                    return push_out(collision, self, other);
                } 
            },
            (ShapeType::Circle(a), ShapeType::Polygon(b)) => {
                if let Some(mut collision) = collision_poly_circle(b, a) {
                    collision.1 = -collision.1;
                    return push_out(collision, self, other);
                }
            }
            (ShapeType::Polygon(a), ShapeType::Circle(b)) => {
                if let Some(collision) = collision_poly_circle(a, b) {
                    return push_out(collision, self, other);
                }
            }
            (ShapeType::Polygon(a), ShapeType::Polygon(b)) => {
                if let Some(collision) = collision_poly_poly(a, b) {
                    return push_out(collision, self, other);
                }
            }   
        }
        None
    }

    pub fn find_contact_points(&self, other: &RigidBody, collision_normal: Vector2f<f64>) -> Vec<Vector2f<f64>> {
        match (&self.shape, &other.shape) {
            (ShapeType::Circle(a), ShapeType::Circle(_)) => vec![a.center + collision_normal * a.radius],
            (ShapeType::Circle(a), ShapeType::Polygon(b)) => contact_poly_circle(b, a),
            (ShapeType::Polygon(a), ShapeType::Circle(b)) => contact_poly_circle(a, b),
            (ShapeType::Polygon(a), ShapeType::Polygon(b)) => contact_poly_poly(a, b),
        }
    }

    pub fn resolve_collision(&mut self, other: &mut RigidBody, collision: &CollisionData, contact_points: Vec<Vector2f<f64>>) {
        let a = self;
        let b = other;

        let a_inv_mass = if a.is_static { 0.0 } else { 1.0 / a.shape.area() };
        let a_inv_inertia = if a.is_static { 0.0 } else { 1.0 / a.shape.momemnt_of_inertia() };
        let b_inv_mass = if b.is_static { 0.0 } else { 1.0 / b.shape.area() };
        let b_inv_inertia = if b.is_static { 0.0 } else { 1.0 / b.shape.momemnt_of_inertia() };

        let CollisionData(_, normal) = *collision;
        let contact_count = contact_points.len() as f64;

        let mut j_values: [f64; 2] = [0.0; 2];
        let a_lin_vel = a.linear_velocity;
        let b_lin_vel = b.linear_velocity;
        let a_ang_vel = a.angular_velocity;
        let b_ang_vel = b.angular_velocity;
        for i in 0..contact_points.len() {
            let ra = contact_points[i] - a.shape.get_center();
            let rb = contact_points[i] - b.shape.get_center();
            let a_contact_vel = a_lin_vel + ra.perpendicular() * a_ang_vel;
            let b_contact_vel = b_lin_vel + rb.perpendicular() * b_ang_vel;
            let relative_velocity = a_contact_vel - b_contact_vel;

            if relative_velocity.dot(normal) < 0.0 { 
                continue; 
            }

            let elasticity = a.elasticity.min(b.elasticity);
            let v_rel = -(1.0 + elasticity) * relative_velocity.dot(normal);
            let denom = a_inv_mass + b_inv_mass + 
                f64::powi(ra.cross(normal), 2) * a_inv_inertia + 
                f64::powi(rb.cross(normal), 2) * b_inv_inertia;

            let j = v_rel / (denom * contact_count);
            j_values[i] = j;

            let a_impulse = normal * j;
            let b_impulse = normal * -j;

            a.linear_velocity += a_impulse * a_inv_mass;
            b.linear_velocity += b_impulse * b_inv_mass;

            a.angular_velocity += ra.cross(a_impulse) * a_inv_inertia;
            b.angular_velocity += rb.cross(b_impulse) * b_inv_inertia;
        }
        
        // Calculate friction
        let sf = (a.static_friction + b.static_friction) / 2.0;
        let df = (a.dynamic_friction + b.dynamic_friction) / 2.0;

        let a_lin_vel = a.linear_velocity;
        let b_lin_vel = b.linear_velocity;
        let a_ang_vel = a.angular_velocity;
        let b_ang_vel = b.angular_velocity;
        for i in 0..contact_points.len() {
            let ra = contact_points[i] - a.shape.get_center();
            let rb = contact_points[i] - b.shape.get_center();
            let a_contact_vel = a_lin_vel + ra.perpendicular() * a_ang_vel;
            let b_contact_vel = b_lin_vel + rb.perpendicular() * b_ang_vel;
            let relative_velocity = a_contact_vel - b_contact_vel;
            
            let mut tangent = relative_velocity - normal * relative_velocity.dot(normal);
            if tangent.nearly_equal(Vector2f::zero(), 0.0005) {
                continue;
            }

            tangent = tangent.normalize();

            let v_rel = -relative_velocity.dot(tangent);
            let denom = a_inv_mass + b_inv_mass +
                f64::powi(ra.cross(tangent), 2) * a_inv_inertia + 
                f64::powi(rb.cross(tangent), 2) * b_inv_inertia;

            let mut jt = v_rel / (denom * contact_count);
            let j = j_values[i];
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
}