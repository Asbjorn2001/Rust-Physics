use core::f64;
use std::collections::HashMap;
use std::path::Path;
use std::vec;
use graphics::math::Matrix2d;
use graphics::rectangle::square;
use graphics::triangulation::{tx, ty};
use graphics::{color, ellipse, line, triangulation, Context, ImageSize, Rectangle, Transformed};
use opengl_graphics::{GlGraphics, Texture};
use piston_window::{texture, TextureSettings};
use rand::distr::Uniform;
use rand::Rng;

use crate::Vector2f;
use crate::game::PhysicsSettings;
use crate::physics::shape_type::ShapeType;
use crate::physics::shape::Shape;
use crate::physics::polygon::Polygon;
use crate::physics::circle::Circle;
use super::material::*;
use super::shape_mesh::TiledMesh;
use crate::Graphics;

use super::collision::*;
use super::shape::Renderable;

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
            let CollisionData(mut sep, normal) = data;    
            sep -= f64::EPSILON;
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

        let push_poly_circle = |p: &mut Polygon, c: &mut Circle| {
            if p.contains_point(c.center) {
                let cp = p.find_closest_point(c.center);
                c.center = cp + (cp - c.center).normalize() * (c.radius + f64::EPSILON);
            }
        };

        match (&mut self.shape, &mut other.shape) {
            (ShapeType::Circle(a), ShapeType::Circle(b)) => {
                if let Some(collision) = collision_circle_circle(a, b) {
                    return push_out(collision, self, other);
                } 
            },
            (ShapeType::Circle(c), ShapeType::Polygon(p)) => {
                push_poly_circle(p, c);
                if let Some(mut collision) = collision_poly_circle(p, c) {
                    collision.1 = -collision.1;
                    return push_out(collision, self, other);
                }
            }
            (ShapeType::Polygon(p), ShapeType::Circle(c)) => {
                push_poly_circle(p, c);
                if let Some(collision) = collision_poly_circle(p, c) {
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

    pub fn resolve_collision(&mut self, other: &mut RigidBody, normal: &Vector2f<f64>, contact_points: &Vec<Vector2f<f64>>) {
        let a = self;
        let b = other;

        // Calculate constants
        let a_inv_mass = if a.is_static { 0.0 } else { 1.0 / (a.shape.area() * a.material.density) };
        let a_inv_inertia = if a.is_static { 0.0 } else { 1.0 / (a.shape.momemnt_of_inertia() * a.material.density) };
        let b_inv_mass = if b.is_static { 0.0 } else { 1.0 / (b.shape.area() * b.material.density) };
        let b_inv_inertia = if b.is_static { 0.0 } else { 1.0 / (b.shape.momemnt_of_inertia() * b.material.density) };

        let restitution = a.material.restitution.min(b.material.restitution);

        let sf = (a.material.static_friction + b.material.static_friction) / 2.0;
        let df = (a.material.dynamic_friction + b.material.dynamic_friction) / 2.0;

        let normal = *normal;
        for contact_point in contact_points {
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
    /*     match &self.shape {
            ShapeType::Circle(circle) => {
                let identity_matrix = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
                let verts = Polygon::new_regular_polygon(
                    circle.radius as u32 * 3, circle.radius, Vector2f::new(0.0, 0.0), color::BLACK).local_vertices;

                let size = 2.0 * circle.radius;

                let tile_width = texture.get_width() as f64;
                let tile_height = texture.get_height() as f64;

                let ratio_x = size / tile_width;
                let ratio_y = size / tile_height;

                let tiles_x = ratio_x.ceil() as usize;
                let tiles_y = ratio_y.ceil() as usize;

                let mut tiles = vec![];
                for i in 0..tiles_x {
                    for j in 0..tiles_y {
                        let tile_min_x = -circle.radius + i as f64 * tile_width;
                        let tile_max_x = tile_min_x + tile_width;
                        let tile_min_y = -circle.radius + j as f64 * tile_height;
                        let tile_max_y = tile_min_y + tile_height;

                        let mut tile_box = vec![
                            Vector2f::new(tile_min_x, tile_min_y),
                            Vector2f::new(tile_max_x, tile_min_y),
                            Vector2f::new(tile_max_x, tile_max_y),
                            Vector2f::new(tile_min_x, tile_max_y)
                        ];

                        suth_hodg_clip(&mut tile_box, &verts);
                        let mut tiles_uv = vec![];
                        let mut tiles_xy = vec![];
                        for v in tile_box {
                            let uv_x = (v.x - tile_min_x) / tile_width; 
                            let uv_y = 1.0 - (v.y - tile_min_y) / tile_height;
                            tiles_uv.push([uv_x, uv_y]);
                            tiles_xy.push([v.x, v.y]);
                        }
                        tiles.push((tiles_xy, tiles_uv));
                    }
                }
                
                for (tiles_xy, tiles_uv) in tiles {
                    let mut positions = vec![];
                    triangulation::stream_polygon_tri_list(identity_matrix, tiles_xy.clone().into_iter(), |f| {
                        for v in f {
                            positions.push(*v);
                        }
                    });

                    let mut uvs = vec![];
                    triangulation::stream_polygon_tri_list(identity_matrix, tiles_uv.into_iter(), |f| {
                        for uv in f {
                            uvs.push(*uv);
                        }
                    });

                    let m = transform.trans_pos(circle.center).rot_rad(circle.rotation);
                    let v: Vec<[f32; 2]> = positions.iter().map(|v| [tx(m, v[0] as f64, v[1] as f64), ty(m, v[0] as f64, v[1] as f64)]).collect();
                    gl.tri_list_uv(&c.draw_state, &color::WHITE, texture, |f| {
                        f(
                            &v,
                            &uvs
                        )
                    });
 
                    for i in 0..tiles_xy.len() {
                        let a = tiles_xy[i];
                        let b = tiles_xy[(i + 1) % tiles_xy.len()];
                        let l = [a[0], a[1], b[0], b[1]];
                        line(color::RED, 1.0, l, c.transform.trans_pos(circle.center).rot_rad(circle.rotation), gl);
                    }
                }
            }
            
            ShapeType::Polygon(p) => {
                let vertices = p.local_vertices.clone();

                let min_x = vertices.iter().map(|v| v.x as i32).min().unwrap();
                let max_x = vertices.iter().map(|v| v.x as i32).max().unwrap();
                let min_y = vertices.iter().map(|v| v.y as i32).min().unwrap();
                let max_y = vertices.iter().map(|v| v.y as i32).max().unwrap();
                
                let width = (max_x - min_x) as f64;
                let height = (max_y - min_y) as f64;

                let tile_width = texture.get_width() as f64;
                let tile_height = texture.get_height() as f64;

                let ratio_x = width / tile_width;
                let ratio_y = height / tile_height;

                let tiles_x = ratio_x.ceil() as usize;
                let tiles_y = ratio_y.ceil() as usize;

                let mut tiles = vec![];
                for i in 0..tiles_x {
                    for j in 0..tiles_y {
                        let tile_min_x = min_x as f64 + i as f64 * tile_width;
                        let tile_max_x = tile_min_x + tile_width;
                        let tile_min_y = min_y as f64 + j as f64 * tile_height;
                        let tile_max_y = tile_min_y + tile_height;

                        let mut tile_box = vec![
                            Vector2f::new(tile_min_x, tile_min_y),
                            Vector2f::new(tile_max_x, tile_min_y),
                            Vector2f::new(tile_max_x, tile_max_y),
                            Vector2f::new(tile_min_x, tile_max_y)
                        ];

                        suth_hodg_clip(&mut tile_box, &vertices);
                        
                        let mut tiles_uv = vec![];
                        let mut tiles_xy = vec![];
                        for v in tile_box {
                            let uv_x = (v.x - tile_min_x) / tile_width; 
                            let uv_y = 1.0 - (v.y - tile_min_y) / tile_height;
                            tiles_uv.push([uv_x, uv_y]);
                            tiles_xy.push([v.x, v.y]);
                        }
                        tiles.push((tiles_xy, tiles_uv));
                    }
                }

                for (tiles_xy, tiles_uv) in tiles {
                    let mut positions = vec![];
                    triangulation::stream_polygon_tri_list(transform.trans_pos(p.center).rot_rad(p.rotation), tiles_xy.clone().into_iter(), |f| {
                        for v in f {
                            positions.push(*v);
                        }
                    });

                    let identity_matrix = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
                    let mut uvs = vec![];
                    triangulation::stream_polygon_tri_list(identity_matrix, tiles_uv.into_iter(), |f| {
                        for uv in f {
                            uvs.push(*uv);
                        }
                    });

                    gl.tri_list_uv(&c.draw_state, &color::WHITE, texture, |f| {
                        f(
                            &positions,
                            &uvs
                        )
                    });
 
                    for i in 0..tiles_xy.len() {
                        let a = tiles_xy[i];
                        let b = tiles_xy[(i + 1) % tiles_xy.len()];
                        let l = [a[0], a[1], b[0], b[1]];
                        line(color::RED, 1.0, l, c.transform.trans_pos(p.center).rot_rad(p.rotation), gl);
                    }
                }    
            }
        }
    }
} */

// Returns the point of intersection between two lines (p1, p2) and (p3, p4)
fn intersection_line_line(p1: Vector2f<f64>, p2: Vector2f<f64>, p3: Vector2f<f64>, p4: Vector2f<f64>) -> Option<Vector2f<f64>> {
    let denom = (p1.x - p2.x) * (p3.y - p4.y) - (p1.y - p2.y) * (p3.x - p4.x);

    if denom.abs() < 1e-10 {
        return None; // Lines are parallel or coincident
    }

    let x = ((p1.x * p2.y - p1.y * p2.x) * (p3.x - p4.x) - (p1.x - p2.x) * (p3.x * p4.y - p3.y * p4.x)) / denom;
    let y = ((p1.x * p2.y - p1.y * p2.x) * (p3.y - p4.y) - (p1.y - p2.y) * (p3.x * p4.y - p3.y * p4.x)) / denom;

    Some(Vector2f::new(x, y))
}

fn is_inside(p: Vector2f<f64>, a: Vector2f<f64>, b: Vector2f<f64>) -> bool {
    let ab = b - a;
    let ap = p - a;
    ap.cross(ab) <= 0.0
}

fn clip(poly: &mut Vec<Vector2f<f64>>, a: Vector2f<f64>, b: Vector2f<f64>) {
    let mut new_poly = vec![];

    for i in 0..poly.len() {
        let current = poly[i];
        let next = poly[(i + 1) % poly.len()];

        let current_inside = is_inside(current, a, b);
        let next_inside = is_inside(next, a, b);

        if current_inside && next_inside {
            new_poly.push(next);
        } else if current_inside && !next_inside {
            if let Some(intersect) = intersection_line_line(current, next, a, b) {
                new_poly.push(intersect);
            }
        } else if !current_inside && next_inside {
            if let Some(intersect) = intersection_line_line(current, next, a, b) {
                new_poly.push(intersect);
            }
            new_poly.push(next);
        }
        // else: both outside, do nothing
    }

    poly.clear();
    poly.extend(new_poly);
}

// Source: https://www.geeksforgeeks.org/polygon-clipping-sutherland-hodgman-algorithm/
fn suth_hodg_clip(subject_polygon: &mut Vec<Vector2f<f64>>, clip_polygon: &Vec<Vector2f<f64>>) {
    for i in 0..clip_polygon.len() {
        let a = clip_polygon[i];
        let b = clip_polygon[(i + 1) % clip_polygon.len()];
        clip(subject_polygon, a, b);
    }
}
 
