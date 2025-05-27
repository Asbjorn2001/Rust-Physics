use core::f64;
use std::{cell::{Ref, RefCell}, f32::EPSILON, rc::Rc};

use graphics::{ellipse, line, math::Matrix2d, rectangle::square, Context};
use opengl_graphics::GlGraphics;
use graphics::color;
use crate::{game::{Game, PhysicsSettings}, Vector2f};
use crate::utils::helpers::*;

use super::{collision::{collision_circle_segment, collision_poly_segment, contact_poly_segment, point_segment_distance, CollisionData}, material::STEEL, rigid_body::RigidBody, shape::Shape, shape_type::ShapeType};

pub struct Attachment {
    pub obj_ref: Rc<RefCell<RigidBody>>,
    pub rel_pos: Vector2f<f64>, 
}

impl Attachment {
    fn get_att_point(&self) -> Vector2f<f64> {
        let obj = self.obj_ref.borrow();
        obj.shape.get_center() + self.rel_pos.rotate(obj.shape.get_rotation())
    }
}

pub struct StringJoint {
    pub position: Vector2f<f64>,
    pub predicted_position: Vector2f<f64>,
    pub velocity: Vector2f<f64>,
    pub mass: f64,
    pub attachment: Option<Attachment>,
}

impl StringJoint {
    fn get_inv_mass(&self) -> f64 {
        if let Some(att) = &self.attachment {
            att.obj_ref.borrow().get_inv_mass()
        } else {
            1.0 / self.mass
        }
    }
}

pub struct StringConstraint {
    pub index_a: usize,
    pub index_b: usize,
    pub rest_length: f64,
    pub tear_length: f64,
    pub stiffness: f64,    
}

pub struct CollisionConstraint {
    pub index_a: usize,
    pub index_b: usize,
    pub contact_point: Vector2f<f64>,
    pub normal: Vector2f<f64>,
    pub object: Rc<RefCell<RigidBody>>,
}

pub struct StringBody {
    pub joints: Vec<StringJoint>,
    pub constraints: Vec<StringConstraint>,        
}

const CONSTRAINT_ITERATIONS: usize = 8;
const BASE_STIFFNESS: f64 = 1.0;
const BASE_REST_LENGTH: f64 = 10.0;
const BASE_JOINT_MASS: f64 = 5.0;
const BASE_TEAR_LENGTH: f64 = 40.0;

impl StringBody {
    pub fn new(start_position: Vector2f<f64>, num_joints: usize) -> Self {
        let rest_length = BASE_REST_LENGTH;
        let tear_length = BASE_TEAR_LENGTH;
        let stiffness = 1.0 - f64::powf(1.0 - BASE_STIFFNESS, 1.0 / CONSTRAINT_ITERATIONS as f64);
        let mass = BASE_JOINT_MASS;

        let mut joints = vec![];
        let mut constraints = vec![];
        for i in 0..num_joints {
            let position = start_position + Vector2f::new(0.0, rest_length) * i as f64;
            let joint = StringJoint {  
                position,
                predicted_position: position,
                velocity: Vector2f::zero(),
                mass,
                attachment: None,
            };
            joints.push(joint);
            
            // if this segment is not the last
            if i < num_joints - 1 {
                let constraint = StringConstraint {
                    index_a: i,
                    index_b: i + 1,
                    rest_length,
                    tear_length,
                    stiffness
                };
                constraints.push(constraint);
            }
        }

        Self { 
            joints, 
            constraints, 
        }
    }

    pub fn resolve_constraints(&mut self, dt: f64, physics: &PhysicsSettings, objects: &Vec<Rc<RefCell<RigidBody>>>, contacts: &mut Vec<Vector2f<f64>>) -> Option<StringBody> {
        for joint in self.joints.as_mut_slice() {
            if let Some(att) = &joint.attachment {
                joint.position = att.get_att_point();
                joint.velocity = att.obj_ref.borrow().linear_velocity;
            } else {
                joint.velocity += physics.gravity * dt;
                joint.velocity *= 1.0 - physics.air_density * dt;
            }

            joint.predicted_position = joint.position + joint.velocity * dt;
        }

        let collision_constraints = self.generate_collision_constraints(objects);
        for c in collision_constraints.as_slice() {
            contacts.push(c.contact_point);
        }

        for _ in 0..CONSTRAINT_ITERATIONS {
            let mut tear_index = None;
            for (i, constraint) in self.constraints.iter().enumerate() {
                let (a, b) = get_pair_mut(&mut self.joints, constraint.index_a, constraint.index_b);
                let rel_pos = b.predicted_position - a.predicted_position;
                let dist = rel_pos.len();
                let stretch = dist - constraint.rest_length;
                if stretch > constraint.tear_length {
                    tear_index = Some(i);
                    break;
                } 
                if stretch > 0.0 {
                    let a_inv_mass = a.get_inv_mass();
                    let b_inv_mass = b.get_inv_mass();
                    let denom = a_inv_mass + b_inv_mass;

                    let normal = rel_pos / dist;
                    a.predicted_position += normal * stretch * constraint.stiffness * a_inv_mass / denom;
                    b.predicted_position += -normal * stretch * constraint.stiffness * b_inv_mass / denom;
                }
            }
            
            if let Some(i) = tear_index {
                return self.tear_string_at(i);
            }

            for constraint in collision_constraints.as_slice() {
                let (a, b) = get_pair_mut(&mut self.joints, constraint.index_a, constraint.index_b);
                let mut obj = constraint.object.borrow_mut();
                let obj_mass = obj.shape.area() * obj.material.density;

                let mut resolve = |p: &mut StringJoint, cp: Vector2f<f64>, normal: Vector2f<f64>| {
                    let rel_pos = p.predicted_position - cp;
                    let depth = rel_pos.dot(normal);
                    if depth < 0.0 {
                        if obj.is_static {
                            p.predicted_position += -normal * depth
                        } else {
                            let sum_mass = obj_mass + p.mass;
                            obj.linear_velocity += normal * depth * p.mass / (sum_mass * dt);
                            p.predicted_position += -normal * depth * obj_mass / sum_mass;
                        }
                    }
                };
                resolve(a, constraint.contact_point, constraint.normal);
                resolve(b, constraint.contact_point, constraint.normal);
            }
        }

        // Integrate velocities
        for joint in self.joints.as_mut_slice() {
            let next_velocity = (joint.predicted_position - joint.position) / dt;
            if let Some(att) = &joint.attachment {
                let mut obj = att.obj_ref.borrow_mut();
                if !obj.is_static {
                    obj.linear_velocity += next_velocity - joint.velocity;
                }
            } else {
                joint.position = joint.predicted_position; 
                joint.velocity = next_velocity;
            }
        }

        self.resolve_collisions(&collision_constraints);

        None
    }

    fn resolve_collisions(&mut self, collision_constraints: &Vec<CollisionConstraint>) {
        for constraint in collision_constraints {
            let mut obj = constraint.object.borrow_mut();
            let normal = constraint.normal;
            let r = constraint.contact_point - obj.shape.get_center();
            let obj_inv_mass = obj.get_inv_mass();
            let obj_inv_inertia = obj.get_inv_inertia();
            let restitution = obj.material.restitution;
            let sf = obj.material.static_friction;
            let df = obj.material.dynamic_friction;

            let mut resolve_collision = |p: &mut StringJoint| {
                let p_inv_mass = p.get_inv_mass();
                let relative_velocity = p.velocity - (obj.linear_velocity + r.perpendicular() * obj.angular_velocity);

                if relative_velocity.dot(normal) < 0.0 {
                    return;
                }
                
                let v_rel = -(1.0 + restitution) * relative_velocity.dot(normal);
                let denom = (p_inv_mass + obj_inv_mass + (f64::powi(r.cross(normal), 2) * obj_inv_inertia)).max(f64::EPSILON);
                let j = v_rel / denom;
                let p_impulse = normal * j;
                let obj_impulse = normal * -j;

                // Apply impulse
                p.velocity += p_impulse * p_inv_mass;
                obj.linear_velocity += obj_impulse * obj_inv_mass;
                obj.angular_velocity += r.cross(obj_impulse) * obj_inv_inertia;

                // Compute friction
                let relative_velocity = p.velocity - (obj.linear_velocity + r.perpendicular() * obj.angular_velocity);
                let mut tangent = relative_velocity - normal * relative_velocity.dot(normal);
                if tangent.nearly_equal(Vector2f::zero(), 0.0005) {
                    return;
                }
                tangent = tangent.normalize();
                
                let v_rel = -relative_velocity.dot(tangent);
                let denom = (p_inv_mass + obj_inv_mass + (f64::powi(r.cross(tangent), 2) * obj_inv_inertia)).max(f64::EPSILON);

                let mut jt = v_rel / denom;
                if jt.abs() > -j * sf {
                    jt = j * df;
                }

                let p_friction_impulse = tangent * jt;
                let obj_friction_impulse = tangent * -jt;

                p.velocity += p_friction_impulse * p_inv_mass;
                obj.linear_velocity += obj_friction_impulse * obj_inv_mass;
                obj.angular_velocity += r.cross(obj_friction_impulse) * obj_inv_inertia; 
            };

            let (a, b) = get_pair_mut(&mut self.joints, constraint.index_a, constraint.index_b);
            resolve_collision(a);
            resolve_collision(b);
        }
    }

    fn tear_string_at(&mut self, i: usize) -> Option<StringBody> {
        if i == self.constraints.len() - 1 {
            self.constraints.pop();
            self.joints.pop();
        } else if i == 0 {
            self.constraints.remove(0);
            self.joints.remove(0);
            self.constraints.iter_mut().for_each(|c| {
                c.index_a -= 1;
                c.index_b -= 1;
            });
        } else {
            self.constraints.remove(i);
            let mut constraints = self.constraints.split_off(i); // safe now
            constraints.iter_mut().enumerate().for_each(|(i, c)| {
                c.index_a = i;
                c.index_b = i + 1;
            });
            return Some(StringBody {
                joints: self.joints.split_off(i + 1),
                constraints,
            });
        }

        return None;
    }

    fn generate_collision_constraints(&self, objects: &Vec<Rc<RefCell<RigidBody>>>) -> Vec<CollisionConstraint> {
        let mut constraints = vec![];
        for obj_ref in objects {
            let obj = obj_ref.borrow();
            for constraint in self.constraints.as_slice() {
                let (a, b) = (&self.joints[constraint.index_a], &self.joints[constraint.index_b]);
                if let Some(att) = &a.attachment {
                    if att.obj_ref.as_ptr() == obj_ref.as_ptr() {
                        continue;
                    }
                } 
                if let Some(att) = &b.attachment {
                    if att.obj_ref.as_ptr() == obj_ref.as_ptr() {
                        continue;
                    }
                } 
                let collision = match &obj.shape {
                    ShapeType::Circle(c) => collision_circle_segment(&c, a.predicted_position, b.predicted_position),
                    ShapeType::Polygon(p) => collision_poly_segment(&p, a.predicted_position, b.predicted_position)
                };

                if let Some(mut collision) = collision {
                    collision.contacts = match &obj.shape {
                        ShapeType::Circle(c) => vec![c.center + collision.normal * c.radius],
                        ShapeType::Polygon(p) => contact_poly_segment(p, a.predicted_position, b.predicted_position),
                    };

                    for cp in collision.contacts {
                        constraints.push(CollisionConstraint {
                            index_a: constraint.index_a,
                            index_b: constraint.index_b,
                            contact_point: cp,
                            normal: collision.normal,
                            object: obj_ref.clone(),
                        });
                    }
                }
            }
        }
        
        constraints
    }

    pub fn draw(&self, transform: Matrix2d, c: Context, gl: &mut GlGraphics) {
        for i in 0..self.joints.len() {
            if i < self.joints.len() - 1 {
                let a = &self.joints[i];
                let b = &self.joints[i + 1];
                let l = [a.position.x, a.position.y, b.position.x, b.position.y];
                line(color::RED, 4.0, l, transform, gl);
                let square = square(a.position.x, a.position.y, 5.0);
                ellipse(color::GREEN, square, transform, gl);
            }
        }
    }
}
