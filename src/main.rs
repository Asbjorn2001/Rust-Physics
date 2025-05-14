extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate kira;

mod utils;
mod shapes;

use utils::vector2f::Vector2f;
use utils::helpers::*;

use shapes::physical_shape::PhysicalShape;
use shapes::circle::Circle;
use shapes::polygon::Polygon;
use shapes::physics::Physics;

use glutin_window::GlutinWindow as Window;
use graphics::*;
use kira::{AudioManager, AudioManagerSettings, DefaultBackend, Tween};
use opengl_graphics::{GlGraphics, OpenGL, GLSL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::{self, WindowSettings};
use piston::{Button, ButtonArgs, ButtonEvent, ButtonState, CursorEvent, Key, MouseCursorEvent, MouseRelativeEvent, Size};
use core::f64;
use std::{clone, vec};


trait Renderable {
    fn draw(&self, c: Context, gl: &mut GlGraphics);
}

/*
pub trait Geometry {
    fn area(&self) -> f64;

    fn contains_point(&self, point: Vector2f<f64>) -> bool;
}

pub trait Physics {
    fn momemnt_of_inertia(&self) -> f64;

    fn update_vectors(&mut self, dt: f64);

    fn resolve_collision_with(&mut self, other: &mut PhysicalShape) -> bool;
    
    fn resolve_border_collision(&mut self, dimensions: Vector2f<f64>) -> bool;
}

#[derive(Clone)]
pub struct Physical<T: Geometry> {
    velocity: Vector2f<f64>,
    angular_velocity: f64,
    elasticity: f64,
    immovable: bool,
    shape: T,
}

struct CollisionData(f64, Vector2f<f64>, Vector2f<f64>);

impl From<Circle> for Physical<Circle> {
    fn from(value: Circle) -> Self {
        Self { 
            velocity: Vector2f::zero(), 
            angular_velocity: 0.0, 
            elasticity: 0.5, 
            immovable: false, 
            shape: value
        }
    }
}

impl From<Polygon> for Physical<Polygon> {
    fn from(value: Polygon) -> Self {
        Self { 
            velocity: Vector2f::zero(), 
            angular_velocity: 0.0, 
            elasticity: 0.5, 
            immovable: false, 
            shape: value
        }
    }
}


#[derive(Clone)]
pub enum PhysicalShape {
    Circle(Physical<Circle>),
    Polygon(Physical<Polygon>)
}

impl Renderable for PhysicalShape {
    fn draw(&self, c: Context, gl: &mut GlGraphics) {
        match self {
            PhysicalShape::Circle(circle) => circle.shape.draw(c, gl),
            PhysicalShape::Polygon(poly) => poly.shape.draw(c, gl),
        }
    }
}

impl Geometry for PhysicalShape {
    fn area(&self) -> f64 {
        match self {
            PhysicalShape::Circle(c) => c.shape.area(),
            PhysicalShape::Polygon(p) => p.shape.area(),
        }
    }   

    fn contains_point(&self, point: Vector2f<f64>) -> bool {
        match self {
            PhysicalShape::Circle(c) => c.shape.contains_point(point),
            PhysicalShape::Polygon(p) => p.shape.contains_point(point)
        }
    }
}

impl Physics for PhysicalShape {
    fn momemnt_of_inertia(&self) -> f64 {
        match self {
            PhysicalShape::Circle(c) => c.momemnt_of_inertia(),
            PhysicalShape::Polygon(p) => p.momemnt_of_inertia(),   
        }
    }

    fn update_vectors(&mut self, dt: f64) {
        match self {
            PhysicalShape::Circle(c) => c.update_vectors(dt),
            PhysicalShape::Polygon(p) => p.update_vectors(dt),
        }
    }

    fn resolve_collision_with(&mut self, other: &mut PhysicalShape) -> bool {
        match self {
            PhysicalShape::Circle(c) => c.resolve_collision_with(other),
            PhysicalShape::Polygon(p) => p.resolve_collision_with(other),
        }
    }

    fn resolve_border_collision(&mut self, dimensions: Vector2f<f64>) -> bool {
        match self {
            PhysicalShape::Circle(c) => c.resolve_border_collision(dimensions),
            PhysicalShape::Polygon(p) => p.resolve_border_collision(dimensions),
        }
    }
}

impl PhysicalShape {
    fn set_velocity(&mut self, velocity: Vector2f<f64>) {
        match self {
            PhysicalShape::Circle(c) => c.velocity = velocity,
            PhysicalShape::Polygon(p) => p.velocity = velocity,
        }
    }

    fn change_color(&mut self, color: [f32; 4]) {
        match self {
            PhysicalShape::Circle(c) => c.shape.color = color,
            PhysicalShape::Polygon(p) => p.shape.color = color,
        }
    }

    fn center_around(&mut self, position: Vector2f<f64>) {
        match self {
            PhysicalShape::Circle(c) => c.shape.center = position,
            PhysicalShape::Polygon(p) => p.shape.center = position,
        }
    }
}

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

#[derive(Clone, Copy)]
pub struct Circle {
    radius: f64,
    center: Vector2f<f64>,
    color: [f32; 4],
}

impl Renderable for Circle {
    fn draw(&self, c: Context, gl: &mut GlGraphics) {
        let center = self.center;
        let square = rectangle::centered_square(center.x, center.y, self.radius);
        
        graphics::ellipse(self.color, square, c.transform, gl);
    }
}

impl Geometry for Circle {
    fn area(&self) -> f64 {
        self.radius * self.radius * PI
    }

    fn contains_point(&self, point: Vector2f<f64>) -> bool {
        (self.center - point).len() < self.radius
    }
}

impl Physics for Physical<Circle> {
    fn momemnt_of_inertia(&self) -> f64 {
        PI * f64::powi(self.shape.radius, 4) / 4.0
    }

    fn update_vectors(&mut self, dt: f64) {
        self.shape.center += self.velocity * dt;
        self.velocity += GRAVITY * dt;        
        self.velocity *= 1.0 - AIR_RESISTANCE * dt;
    }

    fn resolve_collision_with(&mut self, other: &mut PhysicalShape) -> bool {
        match other {
            PhysicalShape::Circle(c) => resolve_collision_circle_circle(self, c),
            PhysicalShape::Polygon(p) => resolve_collision_poly_circle(p, self),
        }
    }

    fn resolve_border_collision(&mut self, dimensions: Vector2f<f64>) -> bool {        
        let center = self.shape.center;
        let velocity = self.velocity;
        let radius = self.shape.radius;
        let mut collision = false;
        if center.x <= radius {
            self.shape.center.x = radius;
            self.velocity.x = -velocity.x * self.elasticity;
            collision = true;
        } else if center.x >= (dimensions.x - radius) {
            self.shape.center.x = dimensions.x - radius;
            self.velocity.x = -velocity.x * self.elasticity;
            collision = true;
        }

        if center.y <= radius {
            self.shape.center.y = radius;
            self.velocity.y = -velocity.y * self.elasticity;
            collision = true;
        }
        else if center.y >= (dimensions.y - radius) {
            self.shape.center.y = dimensions.y - radius;
            self.velocity.y = -velocity.y * self.elasticity;
            collision = true;
        }

        collision
    }
}

impl Circle {
    fn new(center: Vector2f<f64>, radius: f64, color: [f32; 4]) -> Self {
        Self { 
            radius: radius, 
            center: center, 
            color: color, 
        }
    }
}

#[derive(Clone)]
pub struct Polygon {
    local_vertices: Vec<Vector2f<f64>>,
    center: Vector2f<f64>,
    rotation: f64,
    color: [f32; 4],
}

impl Renderable for Polygon {
    fn draw(&self, c: Context, gl: &mut GlGraphics) {
        let verts: Vec<[f64; 2]> = self.get_vertices().iter().map(|&v| v.into()).collect();

        graphics::polygon(self.color, &verts, c.transform, gl);
    }
}

impl Geometry for Polygon {
    fn area(&self) -> f64 {
        let n = self.local_vertices.len();
        let mut sum = 0.0;
        for i in 0..n {
            let p1 = self.local_vertices[i];
            let p2 = self.local_vertices[(i + 1) % n];
            sum += p1.cross(p2);
        }
        sum.abs() / 2.0
    }

    fn contains_point(&self, point: Vector2f<f64>) -> bool {
        let mut pos = 0;
        let mut neg = 0;

        let verts = self.get_vertices();
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

        true
    }
}

impl Physics for Physical<Polygon> {
    fn momemnt_of_inertia(&self) -> f64 {
        let n = self.shape.local_vertices.len();
        let mut I = 0.0; 

        for i in 0..n {
            let p1 = self.shape.local_vertices[i];
            let p2 = self.shape.local_vertices[(i + 1) % n];
            I += p1.cross(p2) * (p1.dot(p1) + p1.dot(p2) + p2.dot(p2));
        }
        
        (I / 12.0).abs()
    }

    fn update_vectors(&mut self, dt: f64) {
        self.shape.center += self.velocity * dt;
        self.velocity += GRAVITY * dt;
        self.velocity *= 1.0 - AIR_RESISTANCE * dt;

        self.shape.rotation += self.angular_velocity * dt;
        self.angular_velocity *= 1.0 - AIR_RESISTANCE * dt;
    }

    fn resolve_collision_with(&mut self, other: &mut PhysicalShape) -> bool {
        match other {
            PhysicalShape::Circle(c) => resolve_collision_poly_circle(self, c),
            PhysicalShape::Polygon(p) => resolve_collision_poly_poly(self, p),
        }
    }

    fn resolve_border_collision(&mut self, dimensions: Vector2f<f64>) -> bool {
        let mut left_vertex = dimensions / 2.0;
        let mut right_vertex = dimensions / 2.0;
        let mut top_vertex = dimensions / 2.0;
        let mut bottom_vertex = dimensions / 2.0;

        for v in self.shape.get_vertices() {
            if v.x < left_vertex.x {
                left_vertex = v;
            } else if v.x > right_vertex.x {
                right_vertex = v
            }

            if v.y < top_vertex.y {
                top_vertex = v;
            } else if v.y > bottom_vertex.y {
                bottom_vertex = v;
            }
        }

        let mut result = None; // (sep, normal) 
        if left_vertex.x < 0.0 {
            result = Some((left_vertex.x, Vector2f::new(1.0, 0.0), left_vertex));
        } else if right_vertex.x > dimensions.x {
            result = Some((dimensions.x - right_vertex.x, Vector2f::new(-1.0, 0.0), right_vertex));
        }

        if top_vertex.y < 0.0 {
            result = Some((top_vertex.y, Vector2f::new(0.0, 1.0), top_vertex));
        } else if bottom_vertex.y > dimensions.y {
            result = Some((dimensions.y - bottom_vertex.y, Vector2f::new(0.0, -1.0), bottom_vertex));
        }

        if let Some((sep, normal, contact_point)) = result {
            let r = contact_point - self.shape.center;
            self.shape.center -= normal * sep;

            let contact_velocity = self.velocity + r.perpendicular() * self.angular_velocity;
            let v_rel = contact_velocity.dot(normal);

            let denom = 1.0 / self.shape.area() + f64::powi(r.cross(normal), 2) / self.momemnt_of_inertia();
            let j = -(1.0 + self.elasticity) * v_rel / denom;
            
            let impulse = normal * j;
            self.velocity += impulse / self.shape.area();
            self.angular_velocity += r.cross(impulse) / self.momemnt_of_inertia();

            return true;
        }

        false
    }
}

impl Polygon {
    fn new_rectangle(center: Vector2f<f64>, width: f64, height: f64, color: [f32; 4]) -> Self {
        let half_width = width / 2.0;
        let half_height = height / 2.0;
        let local_verts = vec![
            Vector2f::new(-half_width, -half_height), // Top left
            Vector2f::new(half_width, -half_height), // Top right
            Vector2f::new(half_width, half_height), // Bottom right
            Vector2f::new(-half_width, half_height), // Bottom left
        ];

        Self { 
            local_vertices: local_verts, 
            center, 
            rotation: 0.0, 
            color, 
        }
    }

    fn new_square(position: Vector2f<f64>, size: f64, color: [f32; 4]) -> Self {
        Self::new_rectangle(position, size, size, color)
    }

    fn new_regular_polygon(n_sides: u8, radius: f64, center: Vector2f<f64>, color: [f32; 4]) -> Self {
        let mut angle = PI * 270.0 / 180.0; // Starting at 270 degrees
        let angle_increment = (2.0 * PI) / n_sides as f64;
        let mut local_verts = vec![];
        for _ in 0..n_sides {
            let x = radius * f64::cos(angle);
            let y = radius * f64::sin(angle);
            local_verts.push(Vector2f::new(x, y));
            angle += angle_increment;
        }

        Self { 
            local_vertices: local_verts, 
            center, 
            rotation: 0.0, 
            color,
        }
    }

    fn new(vertices: Vec<Vector2f<f64>>, center_pos: Vector2f<f64>, color: [f32; 4]) -> Self {
        let center = Self::compute_centroid(&vertices);
        let localized_verts: Vec<Vector2f<f64>> = vertices.iter().map(|&v| v - center).collect();

        Self { 
            local_vertices: localized_verts, 
            center: center_pos, 
            rotation: 0.0, 
            color: color 
        }
    }

    fn get_vertices(&self) -> Vec<Vector2f<f64>> {
        self.local_vertices.iter().map(|v| v.rotate(self.rotation) + self.center).collect()
    }

    fn closest_vertex_to(&self, point: Vector2f<f64>) -> Vector2f<f64> {
        let vertices = self.get_vertices();
        let mut closest_vertex = vertices[0];
        let mut distance = f64::INFINITY;
        for i in 0..vertices.len() {
            let cp = vertices[i];
            let len = (cp - point).len();
            if len < distance {
                closest_vertex = cp;
                distance = len;
            }
        }
        closest_vertex
    }

    fn compute_centroid(vertices: &Vec<Vector2f<f64>>) -> Vector2f<f64> {
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
        sum_center / sum_weight
    }
}
*/

struct Game {
    players: Vec<Player>,
    bodies: Vec<PhysicalShape>,
}

struct Player {
    cursor: Vector2f<f64>,
    target: Option<Vector2f<f64>>,
    projectile: PhysicalShape,
    color: [f32; 4],
}

pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.
    dimensions: Vector2f<f64>,
    cursor: Vector2f<f64>,
    bodies: Vec<PhysicalShape>,
    target: Option<Vector2f<f64>>,
    projectile: PhysicalShape,
    color: [f32; 4],
    audio_manager: AudioManager,
}

impl App {
    fn render(&mut self, args: &RenderArgs) {

        self.dimensions.x = args.window_size[0];
        self.dimensions.y = args.window_size[1];

        self.gl.draw(args.viewport(), |c, gl| {
            graphics::clear(color::WHITE, gl);
            for obj in self.bodies.as_slice() {
                obj.draw(c, gl);
            }

            if let Some(target) = self.target {
                let line = [self.cursor.x, self.cursor.y, target.x, target.y];
                graphics::line(self.color, 1.0, line, c.transform, gl);
                self.projectile.center_around(self.cursor);
                let mut color = self.color;
                color[3] = 0.5; // Transparency
                self.projectile.change_color(color);
                self.projectile.draw(c, gl);
            }
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        for obj in self.bodies.as_mut_slice() {
            obj.update_vectors(args.dt);
        }

        for i in 0..self.bodies.len() {
            for j in (i+1)..self.bodies.len() {
                let (a, b) = get_pair_mut(&mut self.bodies, i, j);
                if a.resolve_collision_with(b) {
                    //let sound_data = StaticSoundData::from_file("./assets/sounds/Billiard-pool-shot.wav").unwrap();
                    //let mut sound = self.audio_manager.play(sound_data).unwrap();
                    //sound.set_volume(-6.0, Tween::default());
                }
            }
        }

        for body in self.bodies.as_mut_slice() {
            body.resolve_border_collision(self.dimensions);
        }
    }

    fn handle_button_events(&mut self, args: &ButtonArgs) {
        let mut change_velocity = |velocity: Vector2f<f64>| {
            match &mut self.bodies[0] {
                PhysicalShape::Polygon(p) => p.velocity = velocity,
                _ => {}
            }
        };

        if args.state == ButtonState::Press {
            match args.button {
                piston::Button::Keyboard(Key::Up) => change_velocity(Vector2f::new(0.0, -150.0)),
                piston::Button::Keyboard(Key::Down) => change_velocity(Vector2f::new(0.0, 150.0)),
                piston::Button::Keyboard(Key::Left) => change_velocity(Vector2f::new(-150.0, 0.0)),
                piston::Button::Keyboard(Key::Right) => change_velocity(Vector2f::new(150.0, 0.0)),

                piston::Button::Keyboard(Key::R) => self.color = color::RED,
                piston::Button::Keyboard(Key::G) => self.color = color::GREEN,
                piston::Button::Keyboard(Key::B) => self.color = color::BLUE,

                piston::Button::Keyboard(Key::D1) => self.projectile = PhysicalShape::Circle(
                    Circle::new(self.cursor, 5.0, self.color).into()),
                piston::Button::Keyboard(Key::D3) => self.projectile = PhysicalShape::Polygon(
                    Polygon::new_regular_polygon(3, 10.0, self.cursor, self.color).into()),
                piston::Button::Keyboard(Key::D4) => self.projectile = PhysicalShape::Polygon(
                    Polygon::new_regular_polygon(4, 15.0, self.cursor, self.color).into()),
                piston::Button::Keyboard(Key::D5) => self.projectile = PhysicalShape::Polygon(
                    Polygon::new_regular_polygon(5, 20.0, self.cursor, self.color).into()),
                piston::Button::Keyboard(Key::D6) => self.projectile = PhysicalShape::Polygon(
                    Polygon::new_regular_polygon(6, 25.0, self.cursor, self.color).into()),
                piston::Button::Keyboard(Key::D7) => self.projectile = PhysicalShape::Polygon(
                    Polygon::new_regular_polygon(7, 30.0, self.cursor, self.color).into()),
                piston::Button::Keyboard(Key::D8) => self.projectile = PhysicalShape::Polygon(
                    Polygon::new_regular_polygon(8, 35.0, self.cursor, self.color).into()),
                piston::Button::Keyboard(Key::D9) => self.projectile = PhysicalShape::Polygon(
                    Polygon::new_regular_polygon(9, 40.0, self.cursor, self.color).into()),
                
                // Mouse button events
                piston::Button::Mouse(piston::MouseButton::Left) => self.target = Some(self.cursor),

                _ => {}
            }
        } else if args.state == ButtonState::Release {
            match args.button {
                piston::Button::Mouse(piston::MouseButton::Left) => self.launch(),
                _ => {},
            }
        }
    }

    fn launch(&mut self) {
        if let Some(target) = self.target {
            let velocity = (target - self.cursor) * 2.0;

            self.projectile.change_color(self.color);
            self.projectile.center_around(self.cursor);
            self.projectile.set_velocity(velocity);
            self.bodies.push(self.projectile.clone());
    
            self.target = None;
        }
    }
}

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create a Glutin window.
    let mut window: Window = WindowSettings::new("piston-game", [500, 500])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let verts = vec![
        Vector2f::new(-15.0, 15.0),
        Vector2f::new(-15.0, 0.0),
        Vector2f::new(0.0, -15.0),
        Vector2f::new(15.0, 0.0),
        Vector2f::new(15.0, 15.0) 
    ];

    let player_body = PhysicalShape::Polygon(Polygon::new(
        verts,
        Vector2f::new(250.0, 250.0), 
        color::PURPLE,
    ).into());

    let manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()).unwrap();

    // Create a new game and run it.
    let mut app = App {
        gl: GlGraphics::new(opengl),
        dimensions: Vector2f { x: 500.0, y: 500.0 },
        cursor: Vector2f::zero(),
        bodies: vec![player_body],
        target: None,
        projectile: PhysicalShape::Circle(Circle::new(Vector2f::zero(), 5.0, color::RED).into()),
        color: color::RED,
        audio_manager: manager,
    };

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.mouse_cursor_args() {
            app.cursor = args.into();
        }

        if let Some(args) = e.button_args() {                
            app.handle_button_events(&args);
        }

        if let Some(args) = e.update_args() {
            app.update(&args);
        }

        if let Some(args) = e.render_args() {
            app.render(&args);
        }
    }
}
