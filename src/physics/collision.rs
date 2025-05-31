use core::f64;
use std::num::NonZeroI16;
use std::vec;

use crate::utils::helpers::nearly_equal;
use crate::Vector2f;
use crate::physics::circle::Circle;
use crate::physics::polygon::Polygon;

// Returns distance squared and cp
pub fn point_segment_distance(p: Vector2f<f64>, a: Vector2f<f64>, b: Vector2f<f64>) -> (f64, Vector2f<f64>) {
    let ab = b - a;
    let ap = p - a;

    let proj = ab.dot(ap);
    let d = proj / ab.len_squared();

    let cp = match d {
        d if d <= 0.0 => a,
        d if d >= 1.0 => b,
        _ => a + ab * d
    };

    ((p - cp).len_squared(), cp)
}


pub fn contact_poly_circle(p: &Polygon, c: &Circle) -> Vec<Vector2f<f64>> {
    let verts = p.get_transformed_vertices();
    let mut min_dist_sq = f64::INFINITY;
    let mut cp = Vector2f::zero();
    for i in 0..verts.len() {
        let va = verts[i];
        let vb = verts[(i + 1) % verts.len()];

        let (dist_sq, contact) = point_segment_distance(c.center, va, vb);

        if dist_sq < min_dist_sq {
            cp = contact;
            min_dist_sq = dist_sq;
        }
    }

    vec![cp]
}

fn find_contacts(a_verts: &Vec<Vector2f<f64>>, b_verts: &Vec<Vector2f<f64>>) -> Vec<Vector2f<f64>> {
    let mut contact_count = 0;
    let mut contact1 = Vector2f::zero();
    let mut contact2 = Vector2f::zero();
    let mut min_dist_sq = f64::INFINITY;

    let mut iteration = |verts1: &Vec<Vector2f<f64>>, verts2: &Vec<Vector2f<f64>>| {
        for p in verts1 {
            for i in 0..verts2.len() {
                let va = verts2[i];
                let vb = verts2[(i + 1) % verts2.len()];

                let (dist_sq, cp) = point_segment_distance(*p, va, vb);
                
                if nearly_equal(dist_sq, min_dist_sq, 0.05) {
                    if !cp.nearly_equal(contact1, 0.05) {
                        contact2 = cp;
                        contact_count = 2;
                    }
                }

                if dist_sq < min_dist_sq {
                    contact1 = cp;
                    contact_count = 1;   
                    min_dist_sq = dist_sq;
                }
            }
        }
    };

    iteration(&a_verts, &b_verts);
    iteration(&b_verts, &a_verts);

    if contact_count > 1 {
        return vec![contact1, contact2];
    } else if contact_count > 0 {
        return vec![contact1];
    }

    return vec![];
}

pub fn contact_poly_poly(a: &Polygon, b: &Polygon) -> Vec<Vector2f<f64>> {
    let a_verts = a.get_transformed_vertices();
    let b_verts = b.get_transformed_vertices();

    find_contacts(&a_verts, &b_verts)
}

pub fn contact_poly_segment(p: &Polygon, a: Vector2f<f64>, b: Vector2f<f64>) -> Vec<Vector2f<f64>> {
    let a_verts = p.get_transformed_vertices();
    let b_verts = vec![a, b];

    find_contacts(&a_verts, &b_verts)
}


#[derive(Clone)]
pub struct CollisionData {
    pub sep_or_t: f64,
    pub normal: Vector2f<f64>,
    pub contacts: Vec<Vector2f<f64>>,
}


pub fn circle_vs_circle(a: &Circle, b: &Circle) -> Option<CollisionData> {
    let delta_dist = b.center - a.center;
    if delta_dist.len() == 0.0 {
        return None;
    }
    let sum_radius = a.radius + b.radius;
    let seperation = delta_dist.len() - sum_radius;
    if seperation < 0.0 {
        let normal = delta_dist.normalize();
        return Some(CollisionData { sep_or_t: seperation, normal, contacts: vec![] });
    }

    None
}

// With normal pointing towards segment
pub fn circle_vs_segment(c: &Circle, a: Vector2f<f64>, b: Vector2f<f64>) -> Option<CollisionData> {
    let ab = b - a;
    let ac = c.center - a;

    let t = ac.dot(ab) / ab.dot(ab);
    let t = t.clamp(0.0, 1.0);

    let deepest_point = a + ab * t;
    let cd = deepest_point - c.center;
    let dist_sq = cd.len_squared();

    if dist_sq <= c.radius * c.radius { 
        let seperation = f64::sqrt(dist_sq) - c.radius; 
        let normal = cd.normalize();
        Some(CollisionData { sep_or_t: seperation, normal, contacts: vec![c.center + normal * c.radius] }) 
    } else {
        None
    }
}

fn find_min_seperation(a_verts: &Vec<Vector2f<f64>>, b_verts: &Vec<Vector2f<f64>>) -> Option<CollisionData> {
    let mut result = CollisionData { sep_or_t: f64::NEG_INFINITY, normal: Vector2f::zero(), contacts: vec![] };
    for i in 0..a_verts.len() {
        let edge = a_verts[i] - a_verts[(i + 1) % a_verts.len()];
        let normal = edge.perpendicular().normalize();
        let mut min_sep = f64::INFINITY;

        for vb in b_verts {
            let sep = (*vb - a_verts[i]).dot(normal);
            min_sep = min_sep.min(sep);
        }

        if min_sep > 0.0 {
            return None;
        }

        if min_sep > result.sep_or_t {
            result.sep_or_t = min_sep;
            result.normal = normal;
        }
    }

    Some(result)
}

// normal always points towards b
pub fn polygon_vs_polygon(a: &Polygon, b: &Polygon) -> Option<CollisionData> {
    let a_verts = a.get_transformed_vertices();
    let b_verts = b.get_transformed_vertices();
    
    if let Some(a_res) = find_min_seperation(&a_verts, &b_verts) {
        if let Some(mut b_res) = find_min_seperation(&b_verts, &a_verts) {
            b_res.normal = -b_res.normal; 
            return if a_res.sep_or_t > b_res.sep_or_t { Some(a_res) } else { Some(b_res) };
        }
    }

    None
}

// normal always points towards the segment
pub fn polygon_vs_segment(p: &Polygon, a: Vector2f<f64>, b: Vector2f<f64>) -> Option<CollisionData> {
    let poly_verts = p.get_transformed_vertices();
    let segment = vec![a, b];

    if let Some(poly_res) = find_min_seperation(&poly_verts, &segment) {
        if let Some(mut seg_res) = find_min_seperation(&segment, &poly_verts) {
            seg_res.normal = -seg_res.normal; 
            let res = if poly_res.sep_or_t > seg_res.sep_or_t { poly_res } else { seg_res };
            let contacts = contact_poly_segment(p, a, b);
            return Some(CollisionData { contacts, ..res });
        }
    }

    None
}

pub fn polygon_vs_circle(p: &Polygon, c: &Circle) -> Option<CollisionData> {
    let mut result = CollisionData { sep_or_t: f64::NEG_INFINITY, normal: Vector2f::zero(), contacts: vec![] };
    let poly_verts = p.get_transformed_vertices();
    let mut closest_point = Vector2f::zero();
    let mut distance = f64::INFINITY;
    for i in 0..poly_verts.len() {
        let a = poly_verts[i];
        let b = poly_verts[(i + 1) % poly_verts.len()];
        let normal = (a - b).perpendicular().normalize();
        let sep = (c.center - a).dot(normal) - c.radius;

        if sep > 0.0 {
            return None
        }

        if sep > result.sep_or_t {
            result.sep_or_t = sep;
            result.normal = normal;
        }

        let (dist, cp) = point_segment_distance(c.center, a, b);
        if dist < distance {
            closest_point = cp;
            distance = dist;
        }
    }

    let normal = (c.center - closest_point).normalize();
    let sep = (c.center - closest_point).dot(normal) - c.radius;
    if sep > 0.0 {
        return None
    }

    if sep > result.sep_or_t {
        result.sep_or_t = sep;
        result.normal = normal;
    }

    Some(result)
}

// =======================
// Ray collision detection
// =======================

fn ray_intersect_circle(ray_origin: Vector2f<f64>, ray_dir: Vector2f<f64>, center: Vector2f<f64>, radius: f64) -> Option<f64> {
    let oc = ray_origin - center;

    let a = ray_dir.dot(ray_dir);
    let b = 2.0 * oc.dot(ray_dir);
    let c = oc.dot(oc) - radius * radius;

    let discriminant = b * b - 4.0 * a * c;

    if discriminant < 0.0 {
        None
    } else {
        let sqrt_disc = discriminant.sqrt();
        let t1 = (-b - sqrt_disc) / (2.0 * a);
        let t2 = (-b + sqrt_disc) / (2.0 * a);

        // We're only interested in intersections in front of the ray origin (t >= 0)
        let t = if t1 >= 0.0 {
            t1
        } else if t2 >= 0.0 {
            t2
        } else {
            return None;
        };

        Some(t)
    }
}

pub fn ray_vs_circle(ray_origin: Vector2f<f64>, ray_dir: Vector2f<f64>, c: &Circle) -> Option<CollisionData> {
    if let Some(t) = ray_intersect_circle(ray_origin, ray_dir, c.center, c.radius) {
        if t <= 1.0 {
            let cp = ray_origin + ray_dir * t;
            let normal = (cp - c.center).normalize();
            Some(CollisionData { sep_or_t: t, normal, contacts: vec![cp] })
        } else {
            None
        }
    } else {
        None
    }
}

fn ray_intersect_capsule(ray_origin: Vector2f<f64>, ray_dir: Vector2f<f64>, a: Vector2f<f64>, b: Vector2f<f64>, radius: f64) -> Option<f64> {
    let t1 = ray_intersect_circle(ray_origin, ray_dir, a, radius);
    let t2 = ray_intersect_circle(ray_origin, ray_dir, b, radius);

    let seg = b - a;
    let normal = seg.perpendicular().normalize();
    let d_proj = ray_dir.dot(normal);
    if d_proj.abs() < 1e-6 {
        return t1.or(t2);
    }

    let dist = (a - ray_origin).dot(normal);
    let t_wall = (dist - radius) / d_proj;

    if t_wall >= 0.0 {
        let hit_point = ray_origin + ray_dir * t_wall;
        let seg_len = seg.len();
        let seg_proj = (hit_point - a).dot(seg / seg_len);
        if seg_proj >= 0.0 && seg_proj <= seg_len {
            return Some(t_wall.min(t1.unwrap_or(f64::INFINITY)).min(t2.unwrap_or(f64::INFINITY)));
        }
    }

    t1.or(t2)
}

pub fn swept_circle_vs_segment(c: &Circle, ray_dir: Vector2f<f64>, a: Vector2f<f64>, b: Vector2f<f64>) -> Option<CollisionData> {
    if let Some(t) = ray_intersect_capsule(c.center, ray_dir, a, b, c.radius) {
        if t <= 1.0 {
            let mut normal = (a - b).perpendicular().normalize();
            if ray_dir.dot(normal) < 0.0 {
                normal = -normal;
            }
            let contact = c.center + ray_dir + normal * c.radius;
            Some(CollisionData { sep_or_t: t, normal: normal, contacts: vec![contact] })
        } else {
            None
        }
    } else {
        None
    }
}

pub fn swept_polygon_vs_segment(p: &Polygon, ray_dir: Vector2f<f64>, a: Vector2f<f64>, b: Vector2f<f64>) -> Option<CollisionData> {
    let poly_verts = p.get_transformed_vertices();
    let mut min_t = f64::INFINITY;
    let mut corner = Vector2f::zero();
    for vert in poly_verts {
        if let Some(t) = ray_intersect_segment(vert, ray_dir, a, b) {
            if t < min_t {
                min_t = t;
                corner = vert;
            }
        }
    }

    if min_t <= 1.0 {
        let mut normal = (a - b).perpendicular().normalize();
        if normal.dot(ray_dir) < 0.0 {
            normal = -normal;
        }

        return Some(CollisionData { sep_or_t: min_t, normal, contacts: vec![corner + ray_dir] });
    }

    None
}

pub fn ray_vs_polygon(ray_origin: Vector2f<f64>, ray_dir: Vector2f<f64>, p: &Polygon) -> Option<CollisionData> {
    let poly_verts = p.get_transformed_vertices();
    let mut min_t = f64::INFINITY;
    let n = poly_verts.len();
    let mut edge = Vector2f::zero();

    for i in 0..n {
        let a = poly_verts[i];
        let b = poly_verts[(i + 1) % n]; 

        if let Some(t) = ray_intersect_segment(ray_origin, ray_dir, a, b) {
            if t < min_t {
                min_t = t;
                edge = a - b;
            }
        }
    }

    if min_t <= 1.0 {
        let normal = edge.perpendicular().normalize();
        Some(CollisionData { sep_or_t: min_t, normal, contacts: vec![ray_origin + ray_dir * min_t] })
    } else {
        None
    }
}

fn ray_intersect_segment(ray_origin: Vector2f<f64>, ray_dir: Vector2f<f64>, p1: Vector2f<f64>, p2: Vector2f<f64>) -> Option<f64> {
    let v1 = ray_origin - p1;
    let v2 = p2 - p1;
    let v3 = ray_dir.perpendicular();

    let dot = v2.dot(v3);
    if dot.abs() < 1e-6 {
        return None; // Parallel
    }

    let t1 = v2.cross(v1) / dot;
    let t2 = v1.dot(v3) / dot;

    if t1 >= 0.0 && t2 >= 0.0 && t2 <= 1.0 {
        Some(t1)
    } else {
        None
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AABB {
    pub top_left: Vector2f<f64>,
    pub bottom_right: Vector2f<f64>
}

impl AABB {
    pub fn overlap(&self, other: &AABB) -> bool {
        self.top_left.x < other.bottom_right.x &&
        self.bottom_right.x > other.top_left.x &&
        self.top_left.y < other.bottom_right.y &&
        self.bottom_right.y > other.top_left.y
    }

    pub fn width(&self) -> f64 {
        self.bottom_right.x - self.top_left.x
    }

    pub fn height(&self) -> f64 {
        self.bottom_right.y - self.top_left.y
    }

    pub fn expand_by(&self, v: Vector2f<f64>) -> AABB {
        let mut aabb = self.clone();
        if v.x > 0.0 {
            aabb.bottom_right.x += v.x;
        }  else {
            aabb.top_left.x += v.x;
        }
        if v.y > 0.0 {
            aabb.bottom_right.y += v.y;
        } else {
            aabb.top_left.y += v.y;
        }

        aabb
    }

    pub fn contains_point(&self, p: Vector2f<f64>) -> bool {
        p.x >= self.top_left.x && p.x <= self.bottom_right.x &&
        p.y >= self.top_left.y && p.y <= self.bottom_right.y
    }
}

pub fn ray_intersects_aabb(ray_origin: Vector2f<f64>, ray_dir: Vector2f<f64>, aabb: &AABB) -> Option<f64> {
    let inv_dir = Vector2f::new(1.0 / ray_dir.x, 1.0 / ray_dir.y);

    let mut txmin = (aabb.top_left.x - ray_origin.x) * inv_dir.x;
    let mut txmax = (aabb.bottom_right.x - ray_origin.x) * inv_dir.x;

    if inv_dir.x < 0.0 {
        std::mem::swap(&mut txmin, &mut txmax);
    }

    let mut tymin = (aabb.top_left.y - ray_origin.y) * inv_dir.y;
    let mut tymax = (aabb.bottom_right.y - ray_origin.y) * inv_dir.y;

    if inv_dir.y < 0.0 {
        std::mem::swap(&mut tymin, &mut tymax);
    }

    if (txmin > tymax) || (tymin > txmax) {
        return None;
    }

    txmin = txmin.max(tymin);
    txmax = txmax.min(tymax);

    if txmax < 0.0 {
        return None; // AABB is behind the ray
    }

    Some(txmin.max(0.0)) // Return distance to intersection
}