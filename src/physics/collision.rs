use core::f64;

use crate::utils::helpers::nearly_equal;
use crate::Vector2f;
use crate::physics::circle::Circle;
use crate::physics::polygon::Polygon;

// Returns distance squared and cp
fn point_segment_distance(p: Vector2f<f64>, a: Vector2f<f64>, b: Vector2f<f64>) -> (f64, Vector2f<f64>) {
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
    let verts = p.get_vertices();
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


pub fn contact_poly_poly(a: &Polygon, b: &Polygon) -> Vec<Vector2f<f64>> {
    let mut contact_count = 0;
    let mut contact1 = Vector2f::zero();
    let mut contact2 = Vector2f::zero();
    
    let mut min_dist_sq = f64::INFINITY;

    let mut iteration = |verts1: &Vec<Vector2f<f64>>, verts2: &Vec<Vector2f<f64>>, swap: bool| {
        for p in verts1 {
            for i in 0..verts2.len() {
                let va = verts2[i];
                let vb = verts2[(i + 1) % verts2.len()];

                let (dist_sq, cp) = 
                if swap {
                    point_segment_distance(*p, vb, va)
                } else {
                    point_segment_distance(*p, va, vb)
                };
                
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

    let a_verts = a.get_vertices();
    let b_verts = b.get_vertices();
    
    iteration(&a_verts, &b_verts, false);
    iteration(&b_verts, &a_verts, false);

    if contact_count > 1 {
        return vec![contact1, contact2];
    } else if contact_count > 0 {
        return vec![contact1];
    }

    return vec![];
}


#[derive(Clone, Copy)]
pub struct CollisionData(pub f64, pub Vector2f<f64>);


pub fn collision_circle_circle(a: &Circle, b: &Circle) -> Option<CollisionData> {
    let delta_dist = b.center - a.center;
    let sum_radius = a.radius + b.radius;
    let sep = delta_dist.len() - sum_radius;
    if sep < 0.0 {
        let normal = delta_dist.normalize();
        return Some(CollisionData(sep, normal));
    }

    None
}


fn find_min_seperation(a_verts: &Vec<Vector2f<f64>>, b_verts: &Vec<Vector2f<f64>>) -> Option<CollisionData> {
    let mut result = CollisionData(f64::NEG_INFINITY, Vector2f::new(0.0, 0.0));
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

        if min_sep > result.0 {
            result = CollisionData(min_sep, normal);
        }
    }

    Some(result)
}


pub fn collision_poly_poly(a: &Polygon, b: &Polygon) -> Option<CollisionData> {
    let a_verts = a.get_vertices();
    let b_verts = b.get_vertices();
    
    if let Some(a_res) = find_min_seperation(&a_verts, &b_verts) {
        if let Some(mut b_res) = find_min_seperation(&b_verts, &a_verts) {
            b_res.1 = -b_res.1; // normal always points towards b
            return if a_res.0 > b_res.0 { Some(a_res) } else { Some(b_res) };
        }
    }

    None
}


pub fn collision_poly_circle(p: &Polygon, c: &Circle) -> Option<CollisionData> {
    let mut result = CollisionData(f64::NEG_INFINITY, Vector2f::new(0.0, 0.0));
    let poly_verts = p.get_vertices();
    for i in 0..poly_verts.len() {
        let a = poly_verts[i];
        let b = poly_verts[(i + 1) % poly_verts.len()];
        let normal = (a - b).perpendicular().normalize();
        let sep = (c.center - a).dot(normal) - c.radius;

        if sep > 0.0 {
            return None
        }

        if sep > result.0 {
            result = CollisionData(sep, normal);
        }
    }

    Some(result)
}