use crate::Vector2f;
use crate::shapes::circle::Circle;
use crate::shapes::polygon::Polygon;
use crate::shapes::geometry::Geometry;
use crate::shapes::physics::*;

pub struct CollisionData(pub f64, pub Vector2f<f64>, pub Vector2f<f64>);

pub fn collision_circle_circle(a: &Circle, b: &Circle) -> Option<CollisionData> {
    let delta_dist = a.center - b.center;
    let sum_radius = a.radius + b.radius;
    let sep = delta_dist.len() - sum_radius;
    if sep < 0.0 {
        let normal = delta_dist.normalize();
        let contact_point = b.center + normal * (b.radius + sep / 2.0);
        return Some(CollisionData(sep, normal, contact_point));
    }

    None
}

pub fn resolve_collision_circle_circle(a: &mut Physical<Circle>, b: &mut Physical<Circle>) -> bool {
    if let Some(CollisionData(_, normal, contact)) = collision_circle_circle(&a.shape, &b.shape) {
        let m1 = a.shape.area();
        let m2 = b.shape.area();
        let v1 = a.velocity;
        let v2 = b.velocity;
        
        // Push the objects away from each other
        a.shape.center = contact + normal * a.shape.radius;
        b.shape.center = contact - normal * b.shape.radius;

        // Update velocites
        let delta_dist = a.shape.center - b.shape.center;
        a.velocity = v1 - delta_dist * ((1.0 + a.elasticity) * m2) / (m1 + m2) * (v1 - v2).dot(delta_dist) / f64::powi(delta_dist.len(), 2);
        b.velocity = v2 - delta_dist * ((1.0 + b.elasticity) * m1) / (m1 + m2) * (v2 - v1).dot(delta_dist) / f64::powi(delta_dist.len(), 2);

        return true;
    }

    false
}

fn find_min_seperation(a_verts: &Vec<Vector2f<f64>>, b_verts: &Vec<Vector2f<f64>>) -> Option<CollisionData> {
    let mut result = CollisionData(f64::NEG_INFINITY, Vector2f::new(0.0, 0.0), Vector2f::new(0.0, 0.0));
    let mut pen_vert = a_verts[0];
    for i in 0..a_verts.len() {
        let edge = a_verts[i] - a_verts[(i + 1) % a_verts.len()];
        let normal = edge.perpendicular().normalize();
        let mut min_sep = f64::INFINITY;

        for vb in b_verts {
            let sep = (*vb - a_verts[i]).dot(normal);
            if sep < min_sep {
                pen_vert = *vb;
                min_sep = sep;
            }
        }

        if min_sep > 0.0 {
            return None;
        }

        if min_sep > result.0 {
            result = CollisionData(min_sep, normal, pen_vert + normal * min_sep / 2.0);
        }
    }

    Some(result)
}

pub fn collision_poly_poly(a: &Polygon, b: &Polygon) -> Option<CollisionData> {
    let a_verts = a.get_vertices();
    let b_verts = b.get_vertices();
    
    if let Some(a_res) = find_min_seperation(&a_verts, &b_verts) {
        if let Some(b_res) = find_min_seperation(&b_verts, &a_verts) {
            return if a_res.0 > b_res.0 { Some(a_res) } else { Some(b_res) };
        }
    }

    None
}

pub fn resolve_collision_poly_poly(a: &mut Physical<Polygon>, b: &mut Physical<Polygon>) -> bool {
    if let Some(CollisionData(sep, mut normal, contact)) = collision_poly_poly(&a.shape, &b.shape) {
        // Make sure normal always points towards a
        if normal.dot(b.shape.center - a.shape.center) < 0.0 {
            normal = -normal;
        }

        let relative_velocity = a.velocity - b.velocity;
        let impulse = relative_velocity.dot(normal) / (1.0 / a.shape.area() + 1.0 / b.shape.area());
        let a_impulse = -(1.0 + a.elasticity) * impulse;
        let b_impulse = -(1.0 + b.elasticity) * impulse;

        a.shape.center += normal * sep / 2.0;
        b.shape.center -= normal * sep / 2.0;

        a.velocity += normal * a_impulse / a.shape.area();
        b.velocity -= normal * b_impulse / b.shape.area();

        a.angular_velocity += (contact - a.shape.center).cross(normal * a_impulse) / a.momemnt_of_inertia();
        b.angular_velocity -= (contact - b.shape.center).cross(normal * b_impulse) / b.momemnt_of_inertia();

        return true;
    }

    false
}

pub fn collision_poly_circle(p: &Polygon, c: &Circle) -> Option<CollisionData> {
    let mut result = CollisionData(f64::NEG_INFINITY, Vector2f::new(0.0, 0.0), Vector2f::new(0.0, 0.0));
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
            result = CollisionData(sep, normal, c.center - normal * (c.radius - sep / 2.0));
        }
    }

    Some(result)
}

pub fn resolve_collision_poly_circle(p: &mut Physical<Polygon>, c: &mut Physical<Circle>) -> bool {
    if let Some(CollisionData(sep, mut normal, contact_point)) = collision_poly_circle(&p.shape, &c.shape) {
        // Make sure normal always points towards the polygon
        if (c.shape.center - p.shape.center).dot(normal) < 0.0 {
            normal = -normal;
        }

        p.shape.center += normal * sep / 2.0;
        c.shape.center -= normal * sep / 2.0;

        let relative_velocity = p.velocity - c.velocity;
        let impulse = relative_velocity.dot(normal) / (1.0 / p.shape.area() + 1.0 / c.shape.area());
        let p_impulse = -(1.0 + p.elasticity) * impulse;
        let c_impulse = -(1.0 + c.elasticity) * impulse;

        p.velocity += normal * p_impulse / p.shape.area();
        c.velocity -= normal * c_impulse / c.shape.area();

        p.angular_velocity += (contact_point - p.shape.center).cross(normal * p_impulse) / p.momemnt_of_inertia();

        return true;
    }

    false
}

//pub fn resolve_collision_poly_border
//pub fn resolve_collision_circle_border