use std::{cell::{Ref, RefCell}, f32::EPSILON, rc::Rc};

use graphics::{ellipse, line, math::Matrix2d, rectangle::square, Context};
use opengl_graphics::GlGraphics;
use graphics::color;
use crate::{game::{Game, PhysicsSettings}, Vector2f};
use crate::utils::helpers::*;

use super::{collision::{collision_circle_segment, collision_poly_segment, contact_poly_segment, point_segment_distance, CollisionData}, material::STEEL, rigid_body::RigidBody, shape::Shape, shape_type::ShapeType};

struct PointMass {
    position: Vector2f<f64>,
    next_position: Vector2f<f64>,
    velocity: Vector2f<f64>,
    mass: f64,
}

struct Constraint {
    index_a: usize,
    index_b: usize,
    rest_length: f64,
    //tear_length: f64,
    stiffness: f64,
    equality: bool,
}

pub struct StringBody {
    joints: Vec<PointMass>,
    links: Vec<Constraint>,
    pub head: Option<Rc<RefCell<RigidBody>>>,          
    pub tail: Option<Rc<RefCell<RigidBody>>>,                
}

const BASE_STIFFNESS:f64 = 1.0;
const BASE_REST_LENGTH: f64 = 5.0;
const BASE_JOINT_MASS: f64 = 50.0;

impl StringBody {
    pub fn new(start_position: Vector2f<f64>, num_joints: usize) -> Self {
        let mut joints = vec![];
        let mut links = vec![];
        let rest_length = BASE_REST_LENGTH;

        for i in 0..num_joints {
            let position = start_position + Vector2f::new(0.0, rest_length) * i as f64;
            let joint = PointMass {  
                position,
                next_position: position,
                velocity: Vector2f::zero(),
                mass: BASE_JOINT_MASS,
            };
            joints.push(joint);
            
            // if this segment is not the last
            if i < num_joints - 1 {
                let link = Constraint {
                    index_a: i,
                    index_b: i + 1,
                    rest_length,
                    stiffness: BASE_STIFFNESS,
                    equality: true,
                };
                links.push(link);
            }
        }

        Self { 
            joints, 
            links, 
            head: None,
            tail: None,  
        }
    }

    pub fn collide_with(&mut self, body: &RigidBody) -> Vec<Vector2f<f64>> {
        let mut contacts = vec![];
        for link in self.links.as_slice() {
            let (a, b) = get_pair_mut(&mut self.joints, link.index_a, link.index_b);
            let collision = match &body.shape {
                ShapeType::Circle(c) => collision_circle_segment(&c, a.position, b.position),
                ShapeType::Polygon(p) => collision_poly_segment(&p, a.position, b.position)
            };

            if let Some(CollisionData(sep, normal)) = collision {
                match &body.shape {
                    ShapeType::Circle(c) => contacts.push(c.center + normal * c.radius),
                    ShapeType::Polygon(p) => contacts.extend(contact_poly_segment(p, a.position, b.position)),
                }

                a.position += -normal * (sep - f64::EPSILON);
                //a.velocity += -normal * 2.0 * a.velocity.dot(normal);
                b.position += -normal * (sep - f64::EPSILON);
                //b.velocity += -normal * 2.0 * b.velocity.dot(normal);
            }
        }

        contacts
    }

    pub fn update_vectors(&mut self, dt: f64, physics: &PhysicsSettings) {
        for joint in self.joints.as_mut_slice() {
            joint.velocity = (joint.next_position - joint.position) / dt;
            joint.position = joint.next_position; 

            joint.velocity += physics.gravity * dt;
            joint.velocity *= 1.0 - physics.air_density * dt;

            joint.next_position += joint.velocity * dt;
        }
    }

    pub fn update_2(&mut self) {
        if let Some(ptr) = &mut self.head {
            let first_joint = &mut self.joints[0];
            let mut body = ptr.borrow_mut();
            let body_mass = body.material.density * body.shape.area();

            let rel_pos = body.shape.find_closest_point(first_joint.next_position) - first_joint.next_position;
            let stretch = (rel_pos.len() - BASE_REST_LENGTH);
            if stretch > 0.0 {
                let sum_mass = body_mass + first_joint.mass;
                let dir = rel_pos.normalize();
                first_joint.next_position += dir * stretch * body_mass / sum_mass;
                body.shape.translate(-dir * stretch * first_joint.mass / sum_mass);
            }
        }

        if let Some(ptr) = &mut self.tail {
            let last_joint = self.joints.last_mut().unwrap();
            let mut body = ptr.borrow_mut();
            let body_mass = body.material.density * body.shape.area();

            let rel_pos = body.shape.find_closest_point(last_joint.next_position) - last_joint.next_position;
            let stretch = (rel_pos.len() - BASE_REST_LENGTH);
            if stretch > 0.0 {
                let sum_mass = body_mass + last_joint.mass;
                let dir = rel_pos.normalize();
                last_joint.next_position += dir * stretch * body_mass / sum_mass;
                body.shape.translate(-dir * stretch * last_joint.mass / sum_mass);
            }
        }

        for link in self.links.as_slice() {
            let (a, b) = get_pair_mut(&mut self.joints, link.index_a, link.index_b);

            let rel_pos = b.next_position - a.next_position;
            let stretch = rel_pos.len() - link.rest_length;
            if stretch > 0.0 {
                let dir = rel_pos.normalize();
                a.next_position += dir * stretch * 0.5;
                b.next_position += -dir * stretch * 0.5;
            }
        }
    }

    pub fn update(&mut self, dt: f64, game: &Game) {
        self.update_vectors(dt, &game.settings.physics);

        let mut collision_constraints = vec![];
        for joint in self.joints.as_slice() {
            collision_constraints.extend(self.generate_collision_constraints(joint, &game.bodies));
        }

        let iterations = 5;
        for _ in 0..iterations {
            for constraint in self.links.as_slice() {
                Self::project_constraint(&mut self.joints, constraint);
            }
        }
    }

    fn resolve_constraints(&mut self, game: &Game) {
        for joint in self.joints.as_mut_slice() {

        }
    }

    fn generate_collision_constraints(&self, joint: &PointMass, objects: &Vec<Rc<RefCell<RigidBody>>>) -> Vec<Constraint> {
        vec![]
    }

    fn project_constraint(joints: &mut Vec<PointMass>, constraint: &Constraint) {
        let (a, b) = get_pair_mut(joints, constraint.index_a, constraint.index_b);

        let rel_pos = b.position - a.position;
        let dist = rel_pos.len();
        let stretch = dist - constraint.rest_length;
        if stretch > 0.0 {
            let sum_weights = a.mass + b.mass;

            let normal = rel_pos / dist;
            a.next_position += normal * stretch * b.mass / sum_weights;
            b.next_position += -normal * stretch * a.mass / sum_weights;
        }
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

        if let Some(ptr) = &self.head {
            let first_joint = &self.joints[0];
            let body = ptr.borrow();
            let pos = body.shape.get_center();
            let l = [pos.x, pos.y, first_joint.position.x, first_joint.position.y];
            line(color::RED, 4.0, l, transform, gl);
        }

        if let Some(ptr) = &self.tail {
            let last_joint = self.joints.last().unwrap();
            let body = ptr.borrow();
            let pos = body.shape.get_center();
            let l = [pos.x, pos.y, last_joint.position.x, last_joint.position.y];
            line(color::RED, 4.0, l, transform, gl);
        }
    }
}
