use core::f64;
use std::f64::consts::PI;
use graphics::math::Matrix2d;

use crate::Vector2f;
use crate::physics::shape::Renderable;
use crate::GlGraphics;
use crate::physics::shape::Shape;

use super::collision::AABB;

#[derive(Clone)]
pub struct Polygon {
    pub local_vertices: Vec<Vector2f<f64>>,
    pub center: Vector2f<f64>,
    pub rotation: f64,
    area: f64,
    intertia: f64,
}

impl Renderable for Polygon {
    fn draw(&self, transform: Matrix2d, gl: &mut GlGraphics, color: [f32; 4]) {
        let verts: Vec<[f64; 2]> = self.get_transformed_vertices().iter().map(|&v| v.into()).collect();

        graphics::polygon(color, &verts, transform, gl);
    }
}

impl Shape for Polygon {
    fn area(&self) -> f64 {
        return self.area;
    }

    fn momemnt_of_inertia(&self) -> f64 {
        return self.intertia;
    }

    fn get_aabb(&self) -> AABB {
        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY; 
        for v in self.get_transformed_vertices() {
            min_x = min_x.min(v.x);
            max_x = max_x.max(v.x);
            min_y = min_y.min(v.y);
            max_y = max_y.max(v.y);
        }

        return AABB { top_left: Vector2f::new(min_x, min_y), bottom_right: Vector2f::new(max_x, max_y) };
    }

    fn contains_point(&self, point: Vector2f<f64>) -> bool {
        let mut pos = 0;
        let mut neg = 0;

        let verts = self.get_transformed_vertices();
        for i in 0..verts.len() {
            let v1 = verts[i];
            let v2 = verts[(i + 1) % verts.len()];

            let d = (point.x - v1.x) * (v2.y - v1.y) - (point.y - v1.y) * (v2.x - v1.x);

            if d > 0.0 { pos += 1; }
            if d < 0.0 { neg += 1; }

            if pos > 0 && neg > 0 {
                return false;
            }
        }

        return true;
    }

    fn find_closest_surface_point(&self, point: Vector2f<f64>) -> (Vector2f<f64>, Vector2f<f64>) {
        let verts = self.get_transformed_vertices();
        let mut edge = Vector2f::zero();
        let mut closest_point = Vector2f::zero();
        let mut distance = f64::INFINITY;
        for i in 0..verts.len() {
            let a = verts[i];
            let b = verts[(i + 1) % verts.len()];

            let (dist, cp) = super::collision::point_segment_distance(point, a, b);
            if dist < distance {
                closest_point = cp;
                distance = dist;
                edge = a - b;
            }
        }

        return (closest_point, edge.perpendicular().normalize());
    }
}

impl Polygon {
    pub fn new_rectangle(center: Vector2f<f64>, width: f64, height: f64, rotation: f64) -> Self {
        let half_width = width / 2.0;
        let half_height = height / 2.0;
        let local_verts = vec![
            Vector2f::new(-half_width, -half_height), // Top left
            Vector2f::new(half_width, -half_height), // Top right
            Vector2f::new(half_width, half_height), // Bottom right
            Vector2f::new(-half_width, half_height), // Bottom left
        ];

        Self { 
            area: Self::compute_area(&local_verts),
            intertia: Self::compute_momemnt_of_inertia(&local_verts),
            local_vertices: local_verts, 
            center, 
            rotation,
        }
    }

    #[allow(dead_code)]
    pub fn new_square(position: Vector2f<f64>, size: f64, rotation: f64) -> Self {
        Self::new_rectangle(position, size, size, rotation)
    }

    pub fn new_regular_polygon(n_sides: u32, radius: f64, center: Vector2f<f64>, rotation: f64) -> Self {
        let mut angle = PI * 270.0 / 180.0; // Starting at 270 degrees
        let angle_increment = (2.0 * PI) / n_sides as f64;
        if n_sides % 2 == 0 { angle += angle_increment / 2.0; }
        let mut local_verts = vec![];
        for _ in 0..n_sides {
            let x = radius * f64::cos(angle);
            let y = radius * f64::sin(angle);
            local_verts.push(Vector2f::new(x, y));
            angle += angle_increment;
        }

        Self { 
            area: Self::compute_area(&local_verts),
            intertia: Self::compute_momemnt_of_inertia(&local_verts),
            local_vertices: local_verts, 
            center, 
            rotation, 
        }
    }

    pub fn new(vertices: Vec<Vector2f<f64>>, center_pos: Vector2f<f64>, rotation: f64) -> Self {
        let center = Self::compute_center(&vertices);
        let localized_verts: Vec<Vector2f<f64>> = vertices.iter().map(|&v| v - center).collect();

        Self { 
            area: Self::compute_area(&localized_verts),
            intertia: Self::compute_momemnt_of_inertia(&localized_verts),
            local_vertices: localized_verts, 
            center: center_pos, 
            rotation, 
            
        }
    }

    pub fn get_transformed_vertices(&self) -> Vec<Vector2f<f64>> {
        return self.local_vertices.iter().map(|v| v.rotate(self.rotation) + self.center).collect();
    }

    fn compute_center(vertices: &Vec<Vector2f<f64>>) -> Vector2f<f64> {
        let mut sum_center: Vector2f<f64> = Vector2f::zero();
        let mut sum_weight = 0.0;
        let n = vertices.len();
        for i in 0..n {
            let (prev, curr, next) = 
            match i {
                i if i == 0 => (vertices[n - 1], vertices[i], vertices[i + 1]),
                i if i == n - 1 => (vertices[i - 1], vertices[i], vertices[0]),
                i => (vertices[i - 1], vertices[i], vertices[i + 1]),
            };
            let weight = (curr - next).len() + (curr - prev).len();
            sum_center += curr * weight;
            sum_weight += weight;
        }
        
        return sum_center / sum_weight;
    }

    fn compute_area(vertices: &Vec<Vector2f<f64>>) -> f64 {
        let n = vertices.len();
        let mut sum = 0.0;
        for i in 0..n {
            let p1 = vertices[i];
            let p2 = vertices[(i + 1) % n];
            sum += p1.cross(p2);
        }
        return sum.abs() / 2.0;
    }

    fn compute_momemnt_of_inertia(vertices: &Vec<Vector2f<f64>>) -> f64 {
        let n = vertices.len();
        let mut intertia = 0.0; 

        for i in 0..n {
            let p1 = vertices[i];
            let p2 = vertices[(i + 1) % n];
            intertia += p1.cross(p2) * (p1.dot(p1) + p1.dot(p2) + p2.dot(p2));
        }
        
        return (intertia / 12.0).abs();
    }
}