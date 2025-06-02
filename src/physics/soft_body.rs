use core::f64;
use std::{cell::{RefCell}, rc::Rc};

use graphics::{ellipse, line, math::Matrix2d, rectangle::square, Context};
use opengl_graphics::GlGraphics;
use graphics::color;
use crate::{game::{ContactDebug, PhysicsData}, Vector2f};
use crate::utils::helpers::*;
use super::collision::*;
use super::{rigid_body::RigidBody, shape::Shape, shape_type::ShapeType};

// The soft body string is implemented based on Position Based Dynamics 
// source: https://matthias-research.github.io/pages/publications/posBasedDyn.pdf

#[derive(Clone)]
pub struct Attachment {
    pub obj_ref: Rc<RefCell<RigidBody>>,
    pub rel_pos: Vector2f<f64>, 
}

impl Attachment {
    pub fn get_attachment_point(&self) -> Vector2f<f64> {
        let obj = self.obj_ref.borrow();
        obj.shape.get_center() + self.rel_pos.rotate(obj.shape.get_rotation())
    }
}

#[derive(Clone)]
pub struct Joint {
    pub position: Vector2f<f64>,
    pub predicted_position: Vector2f<f64>,
    pub velocity: Vector2f<f64>,
    pub mass: f64,
    pub attachment: Option<Attachment>,
}

impl Joint {
    pub fn new(position: Vector2f<f64>, attachment: Option<Attachment>) -> Self {
        Self { 
            position, 
            predicted_position: position, 
            velocity: Vector2f::zero(), 
            mass: BASE_JOINT_MASS, 
            attachment, 
        }
    }

    fn get_inv_mass(&self) -> f64 {
        if let Some(attachment) = &self.attachment {
            attachment.obj_ref.borrow().get_inv_mass()
        } else {
            1.0 / self.mass
        }
    }
}

#[derive(Clone, Copy)]
pub struct Constraint {
    pub index_a: usize,
    pub index_b: usize,
    pub rest_length: f64,
    pub tear_length: f64,
    pub stiffness: f64,    
}

#[derive(Clone)]
pub struct CollisionConstraint {
    pub index: usize,
    pub contact_point: Vector2f<f64>,
    pub normal: Vector2f<f64>,
    pub object: Rc<RefCell<RigidBody>>,
}

#[derive(Clone)]
pub struct SoftBody {
    pub joints: Vec<Joint>,
    pub constraints: Vec<Constraint>,
    pub damping: f64,
}

const BASE_DAMPING: f64 = 0.2;
const CONSTRAINT_ITERATIONS: usize = 8;
const BASE_STIFFNESS: f64 = 0.9;
#[allow(dead_code)]
const BASE_REST_LENGTH: f64 = 10.0;
const BASE_JOINT_MASS: f64 = 50.0;
#[allow(dead_code)]
const BASE_TEAR_LENGTH: f64 = 100.0;

impl From<Vec<Joint>> for SoftBody {
    fn from(joints: Vec<Joint>) -> Self {
        let stiffness = 1.0 - f64::powf(1.0 - BASE_STIFFNESS, 1.0 / CONSTRAINT_ITERATIONS as f64);
        let mut constraints = vec![];

        if joints.len() >= 2 {
            for i in 0..joints.len() - 1 {
                let rest_length = (joints[i + 1].position - joints[i].position).len();
                constraints.push(Constraint {
                    index_a: i,
                    index_b: i + 1,
                    rest_length,
                    tear_length: rest_length * 2.0,
                    stiffness,
                });
            }
        }
        
        Self { 
            joints,
            constraints, 
            damping: BASE_DAMPING 
        }
    }
}

#[allow(dead_code)]
impl SoftBody {
    pub fn new_string(start_position: Vector2f<f64>, end_position: Vector2f<f64>, num_joints: usize) -> Self {
        let rel_pos = end_position - start_position;
        let dir = rel_pos.normalize();
        let length = rel_pos.len();

        let rest_length = length / (num_joints - 1) as f64;
        let tear_length = 2.0 * rest_length;

        let stiffness = 1.0 - f64::powf(1.0 - BASE_STIFFNESS, 1.0 / CONSTRAINT_ITERATIONS as f64);
        let mass = BASE_JOINT_MASS;

        let mut joints = vec![];
        let mut constraints = vec![];
        for i in 0..num_joints {
            let position = start_position + (dir * rest_length * i as f64);
            let joint = Joint {  
                position,
                predicted_position: position,
                velocity: Vector2f::zero(),
                mass,
                attachment: None,
            };
            joints.push(joint);
            
            // if this segment is not the last
            if i < num_joints - 1 {
                let constraint = Constraint {
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
            damping: BASE_DAMPING,
        }
    }

    pub fn add_joint(&mut self, position: Vector2f<f64>, attachment: Option<Attachment>) {
        let new_joint = Joint {
            position,
            predicted_position: position,
            velocity: Vector2f::zero(),
            mass: BASE_JOINT_MASS,
            attachment,
        };

        let stiffness = 1.0 - f64::powf(1.0 - BASE_STIFFNESS, 1.0 / CONSTRAINT_ITERATIONS as f64);
        let n = self.joints.len();
        let rest_length = (self.joints[n - 1].position - position).len();
        let new_constraint = Constraint {
            index_a: n - 1,
            index_b: n,
            rest_length,
            tear_length: rest_length * 2.0,
            stiffness,
        };

        self.joints.push(new_joint);
        self.constraints.push(new_constraint);
    }

    fn damp_velocities(&mut self) {
        let mut joints_and_masses = vec![];
        for joint in self.joints.as_mut_slice() {
            let mass = if let Some(att) = &joint.attachment {
                let obj = att.obj_ref.borrow();
                obj.shape.area() * obj.material.density
            } else {
                joint.mass
            };
            joints_and_masses.push((joint, mass));
        }

        let mut total_mass = 0.0;
        let mut cm_pos = Vector2f::zero();
        let mut cm_vel = Vector2f::zero();
        for (joint, mass) in joints_and_masses.as_slice() {
            total_mass += mass;
            cm_pos += joint.position * *mass;
            cm_vel += joint.velocity * *mass;
        }
        cm_pos /= total_mass;
        cm_vel /= total_mass;

        let mut angular_momentum = 0.0;
        let mut inertia = 0.0;
        for (joint, mass) in joints_and_masses {
            let r = joint.position - cm_pos;
            angular_momentum += r.cross(joint.velocity * mass);
            inertia += r.dot(r) * mass;
        }

        let angular_velocity = angular_momentum / inertia;
        for joint in self.joints.as_mut_slice() {
            let r = joint.position - cm_pos;
            let dv = cm_vel + r.perpendicular() * angular_velocity - joint.velocity;
            joint.velocity += dv * self.damping;
        }
    }

    pub fn resolve_constraints(
        &mut self, 
        physics: &PhysicsData, 
        objects: &Vec<Rc<RefCell<RigidBody>>>, 
        contacts: &mut Vec<ContactDebug>
    ) -> Option<SoftBody> {
        let dt = physics.dt;
        for joint in self.joints.as_mut_slice() {
            if let Some(att) = &joint.attachment {
                joint.position = att.get_attachment_point();
                joint.velocity = att.obj_ref.borrow().linear_velocity;
            } else {
                joint.velocity += physics.gravity * dt;
                joint.velocity *= 1.0 - physics.air_density * dt;
            }
        }

        self.damp_velocities();

        for joint in self.joints.as_mut_slice() {
            joint.predicted_position = joint.position + joint.velocity * dt;
        }

        let collision_constraints = self.generate_collision_constraints(dt, objects);
        for c in collision_constraints.as_slice() {
            contacts.push(ContactDebug { contact: c.contact_point, normal: c.normal });
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
                let normal = constraint.normal;
                let joint = &mut self.joints[constraint.index];
                let joint_inv_mass = joint.get_inv_mass();

                let mut obj = constraint.object.borrow_mut();
                let obj_inv_mass = obj.get_inv_mass();
                
                let rel_pos = joint.predicted_position - constraint.contact_point;
                let depth = rel_pos.dot(normal);
                if depth < 0.0 {
                    let denom = joint_inv_mass + obj_inv_mass;
                    obj.linear_velocity += normal * depth * obj_inv_mass / (denom * dt);
                    joint.predicted_position += -normal * depth;
                }
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
            let p = &mut self.joints[constraint.index];
            let p_inv_mass = p.get_inv_mass();

            let mut obj = constraint.object.borrow_mut();
            let obj_inv_mass = obj.get_inv_mass();
            let obj_inv_inertia = obj.get_inv_inertia();

            let restitution = obj.material.restitution;
            let sf = obj.material.static_friction;
            let df = obj.material.dynamic_friction;

            let normal = constraint.normal;
            let r = constraint.contact_point - obj.shape.get_center();
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

        }
    }

    fn tear_string_at(&mut self, i: usize) -> Option<SoftBody> {
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
            return Some(SoftBody {
                joints: self.joints.split_off(i + 1),
                constraints,
                damping: self.damping,
            });
        }

        return None;
    }

    pub fn get_aabb(&self) -> AABB {
        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        for joint in self.joints.as_slice() {
            min_x = min_x.min(joint.position.x).min(joint.predicted_position.x);
            max_x = max_x.max(joint.position.x).max(joint.predicted_position.x);
            min_y = min_y.min(joint.position.y).min(joint.predicted_position.y);
            max_y = max_y.max(joint.position.y).max(joint.predicted_position.y);
        }

        AABB { top_left: Vector2f::new(min_x, min_y), bottom_right: Vector2f::new(max_x, max_y)}
    }

    fn generate_collision_constraints(&mut self, dt: f64, objects: &Vec<Rc<RefCell<RigidBody>>>) -> Vec<CollisionConstraint> {
        let mut constraints = vec![];
        let string_aabb = self.get_aabb();
        'obj_loop: for obj_ref in objects {
            let obj = obj_ref.borrow_mut();
            let obj_step = obj.linear_velocity * dt;
            let aabb = obj.shape.get_aabb();

            let mut indices_to_skip = vec![];
            for (i, joint) in self.joints.iter().enumerate() {
                if let Some(attachment) = &joint.attachment {
                    indices_to_skip.push(i);
                    if attachment.obj_ref.as_ptr() == obj_ref.as_ptr() && !obj.is_static {
                        continue 'obj_loop;
                    }
                }

                let rel_vel = obj.linear_velocity - joint.velocity;
                if !aabb.expand_by(rel_vel).overlap(&string_aabb) {
                    indices_to_skip.push(i);
                }
            }
            
            for (i, joint) in self.joints.iter_mut().enumerate() {
                if indices_to_skip.contains(&i) {
                    continue;
                }
                
                let ray_origin = joint.position;
                let collision = if obj.shape.contains_point(ray_origin) {     
                    let (cp, normal) = obj.shape.find_closest_surface_point(joint.predicted_position);   
                    let c_cp = cp - obj.shape.get_center();
                    Some(CollisionData { sep_or_t: (c_cp).len(), normal: normal, contacts: vec![cp] })
                } else {
                    let ray_dir = joint.predicted_position - joint.position - obj_step;
                    match &obj.shape {
                        ShapeType::Circle(c) => ray_vs_circle(ray_origin, ray_dir, c),
                        ShapeType::Polygon(p) => ray_vs_polygon(ray_origin, ray_dir, p),
                    }
                };
                
                if let Some(collision) = collision {
                    constraints.push(CollisionConstraint {
                        index: i,
                        contact_point: collision.contacts[0],
                        normal: collision.normal,
                        object: obj_ref.clone(),
                    });
                }
            }

            for constraint in self.constraints.as_slice() {
                let (i, j) = (constraint.index_a, constraint.index_b);
                if indices_to_skip.contains(&i) || indices_to_skip.contains(&j) {
                    continue;
                }

                let (a, b) = (&self.joints[i], &self.joints[j]);                
                let mut collision = match &obj.shape {
                    ShapeType::Circle(c) => circle_vs_segment(c, a.predicted_position, b.predicted_position),
                    ShapeType::Polygon(p) => polygon_vs_segment(p, a.predicted_position, b.predicted_position),
                };

                // Double check using ray tracing
                if collision.is_none() {
                    let min_dimension = aabb.width().min(aabb.height());
                    if obj.linear_velocity.len_squared() >= min_dimension * min_dimension / 2.0 {
                        collision = match &obj.shape {
                            ShapeType::Circle(c) => swept_circle_vs_segment(c, obj_step, a.predicted_position, b.predicted_position),
                            ShapeType::Polygon(p) => swept_polygon_vs_segment(p, obj_step, a.predicted_position, b.predicted_position),
                        };
                    }
                }

                if let Some(collision) = collision {
                    let joints = vec![i, j];
                    for joint in joints.as_slice() {
                        constraints.push(CollisionConstraint {
                            index: *joint,
                            contact_point: collision.contacts[0],
                            normal: collision.normal,
                            object: obj_ref.clone(),
                        });
                    }
                }
            } 
        }
        
        constraints
    }

    pub fn draw(&self, transform: Matrix2d, _: Context, gl: &mut GlGraphics) {
        for constraint in self.constraints.as_slice() {
            let (a, b) = (&self.joints[constraint.index_a], &self.joints[constraint.index_b]);
            let l = [a.position.x, a.position.y, b.position.x, b.position.y];
            line(color::RED, 2.0, l, transform, gl);
        }

        for joint in self.joints.as_slice() {
            let square = square(joint.position.x - 2.5, joint.position.y - 2.5, 5.0);
            ellipse(color::GREEN, square, transform, gl);
        }
    }
}
