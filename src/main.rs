extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate kira;

use glutin_window::GlutinWindow as Window;
use graphics::math::{separation, translate, Vec2d};
use graphics::*;
use kira::sound::static_sound::StaticSoundData;
use kira::{AudioManager, AudioManagerSettings, DefaultBackend, Tween};
use opengl_graphics::{GlGraphics, OpenGL, GLSL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::{self, WindowSettings};
use piston::{Button, ButtonArgs, ButtonEvent, ButtonState, CursorEvent, Key, MouseCursorEvent, MouseRelativeEvent, Size};
use rand::rand_core::le;
use rand::seq;
use core::f64;
use std::env::Args;
use std::f64::consts::PI;
use std::thread::current;
use std::time::Duration;
use std::vec;

pub mod utils;
use utils::*;

pub mod network;

const EPSILON: f64 = 0.0005;
const GRAVITY: Vector2<f64> = Vector2 { x: 0.0, y: 9.816 };
const ROTATION_DECAY: f64 = 0.99;
const bounciness: f64 = 0.5; // between 0-1

trait Renderable {
    fn draw(&self, c: Context, gl: &mut GlGraphics);
}

trait Physics : Renderable {
    fn area(&self) -> f64;

    fn momemnt_of_inertia(&self) -> f64;

    fn update_vectors(&mut self, dt: f64);

    fn resolve_border_collision(&mut self, dimensions: Vector2<f64>);

    fn contains_point(&self, point: Vector2<f64>) -> bool;
}

#[derive(Clone)]
enum PhysicalBody {
    Circle(Circle),
    ConvexPolygon(ConvexPolygon)
}

impl Renderable for PhysicalBody {
    fn draw(&self, c: Context, gl: &mut GlGraphics) {
        match self {
            PhysicalBody::Circle(circle) => circle.draw(c, gl),
            PhysicalBody::ConvexPolygon(poly) => poly.draw(c, gl),
        }
    }
}

impl Physics for PhysicalBody {
    fn area(&self) -> f64 {
        match self {
            PhysicalBody::Circle(c) => c.area(),
            PhysicalBody::ConvexPolygon(p) => p.area(),
        }
    }   

    fn momemnt_of_inertia(&self) -> f64 {
        match self {
            PhysicalBody::Circle(c) => c.momemnt_of_inertia(),
            PhysicalBody::ConvexPolygon(p) => p.momemnt_of_inertia(),   
        }
    }

    fn update_vectors(&mut self, dt: f64) {
        match self {
            PhysicalBody::Circle(c) => c.update_vectors(dt),
            PhysicalBody::ConvexPolygon(p) => p.update_vectors(dt),
        }
    }

    fn resolve_border_collision(&mut self, dimensions: Vector2<f64>) {
        match self {
            PhysicalBody::Circle(c) => c.resolve_border_collision(dimensions),
            PhysicalBody::ConvexPolygon(p) => p.resolve_border_collision(dimensions),
        }
    }

    fn contains_point(&self, point: Vector2<f64>) -> bool {
        match self {
            PhysicalBody::Circle(c) => c.contains_point(point),
            PhysicalBody::ConvexPolygon(p) => p.contains_point(point)
        }
    }
}

impl PhysicalBody {
    fn resolve_collision_with(&mut self, other: &mut PhysicalBody) -> bool {
        match (self, other) {
            (PhysicalBody::Circle(ca), PhysicalBody::Circle(cb)) => resolve_collision_circle_circle(ca, cb),
            (PhysicalBody::ConvexPolygon(pa), PhysicalBody::ConvexPolygon(pb)) => resolve_collision_poly_poly(pa, pb),
            (PhysicalBody::Circle(c), PhysicalBody::ConvexPolygon(p)) |
            (PhysicalBody::ConvexPolygon(p), PhysicalBody::Circle(c)) => resolve_collision_poly_circle(p, c),
        }
    }

    fn add_velocity(&mut self, velocity: Vector2<f64>) {
        match self {
            PhysicalBody::Circle(c) => c.velocity += velocity,
            PhysicalBody::ConvexPolygon(p) => p.velocity += velocity,
        }
    }

    fn change_color(&mut self, color: [f32; 4]) {
        match self {
            PhysicalBody::Circle(c) => c.color = color,
            PhysicalBody::ConvexPolygon(p) => p.color = color,
        }
    }

    fn center_around(&mut self, position: Vector2<f64>) {
        match self {
            PhysicalBody::Circle(c) => c.center = position,
            PhysicalBody::ConvexPolygon(p) => p.center = position,
        }
    }
}

fn resolve_collision_circle_circle(a: &mut Circle, b: &mut Circle) -> bool {
    let delta_dist = a.center - b.center;
    let sum_radius = a.radius + b.radius;
    if delta_dist.len() < sum_radius {
        let m1 = a.area();
        let m2 = b.area();
        let v1 = a.velocity;
        let v2 = b.velocity;
        
        // Push the objects away from each other
        let a_next = b.center + delta_dist.normalize() * sum_radius;
        b.center = a.center - delta_dist.normalize() * sum_radius;
        a.center = a_next;

        // Update velocites
        a.velocity = v1 - delta_dist * ((1.0 + bounciness) * m2) / (m1 + m2) * (v1 - v2).dot(delta_dist) / f64::powi(delta_dist.len(), 2);
        b.velocity = v2 - delta_dist * ((1.0 + bounciness) * m1) / (m1 + m2) * (v2 - v1).dot(delta_dist) / f64::powi(delta_dist.len(), 2);

        return true;
    }

    false
}

fn resolve_collision_poly_poly(a: &mut ConvexPolygon, b: &mut ConvexPolygon) -> bool {
    let (sep, mut normal, pen_vert) = find_seperation_poly_poly(a, b);
    if sep <= 0.0 {
        // Make sure normal always points towards a
        if normal.dot(b.center - a.center) < 0.0 {
            normal = -normal;
        }

        let relative_velocity = a.velocity - b.velocity;
        let impulse = -(1.0 + bounciness) * relative_velocity.dot(normal) / (1.0 / a.area() + 1.0 / b.area());

        a.center += normal * sep / 2.0;
        b.center -= normal * sep / 2.0;

        a.velocity += normal * impulse / a.area();
        b.velocity -= normal * impulse / b.area();

        let contact_point = pen_vert + normal * sep / 2.0;

        a.angular_velocity += (contact_point - a.center).cross(normal * impulse) / a.momemnt_of_inertia();
        b.angular_velocity += (contact_point - b.center).cross(-normal * impulse) / b.momemnt_of_inertia();

        return true;
    }

    false
}

fn find_seperation_poly_poly(a: &ConvexPolygon, b: &ConvexPolygon) -> (f64, Vector2<f64>, Vector2<f64>) {
    
    fn find_min_seperation(a_verts: &Vec<Vector2<f64>>, b_verts: &Vec<Vector2<f64>>) -> (f64, Vector2<f64>, Vector2<f64>) {
        let mut result = (f64::NEG_INFINITY, Vector2::new(0.0, 0.0), Vector2::new(0.0, 0.0));

        let perpendicular = |vertices: &Vec<Vector2<f64>>, i| -> Vector2<f64> {
            let edge: Vector2<f64> = vertices[i] - vertices[(i + 1) % vertices.len()];
            edge.perpendicular().normalize()
        };

        let mut pen_vertex = a_verts[0];
        for i in 0..a_verts.len() {
            let normal = perpendicular(&a_verts, i);
            let mut min_sep = f64::INFINITY;

            for vb in b_verts {
                let sep = (*vb - a_verts[i]).dot(normal);
                if sep < min_sep {
                    pen_vertex = *vb;
                    min_sep = sep;
                }
            }

            if min_sep > result.0 {
                result = (min_sep, normal, pen_vertex);
            }
        }

        result
    }

    let a_result = find_min_seperation(&a.get_vertices(), &b.get_vertices());
    let b_result = find_min_seperation(&b.get_vertices(), &a.get_vertices());

    if a_result.0 > b_result.0 { a_result } else { b_result }
}

fn resolve_collision_poly_circle(p: &mut ConvexPolygon, c: &mut Circle) -> bool {
    let (sep, mut normal, contact_point) = find_seperation_poly_circle(p, c);
    if sep <= 0.0 {
        // Make sure normal always points towards the polygon
        if (c.center - p.center).dot(normal) < 0.0 {
            normal = -normal;
        }

        p.center += normal * sep / 2.0;
        c.center -= normal * sep / 2.0;

        let relative_velocity = p.velocity - c.velocity;
        let impulse = -(1.0 + bounciness) * relative_velocity.dot(normal) / (1.0 / p.area() + 1.0 / c.area());

        p.velocity += normal * impulse / p.area();
        c.velocity -= normal * impulse / c.area();

        p.angular_velocity += (contact_point - p.center).cross(normal * impulse) / p.momemnt_of_inertia();

        return true;
    }

    false
}

fn find_seperation_poly_circle(p: &ConvexPolygon, c: &Circle) -> (f64, Vector2<f64>, Vector2<f64>) {
    let mut result = (f64::NEG_INFINITY, Vector2::new(0.0, 0.0), Vector2::new(0.0, 0.0));
    let poly_verts = p.get_vertices();
    for i in 0..poly_verts.len() {
        let a = poly_verts[i];
        let b = poly_verts[(i + 1) % poly_verts.len()];
        let normal = (a - b).perpendicular().normalize();
        let sep = (c.center - a).dot(normal) - c.radius;
        if sep > result.0 {
            result = (sep, normal, c.center - normal * c.radius);
        }
    }

    result
}

#[derive(Clone, Copy)]
struct Circle {
    radius: f64,
    center: Vector2<f64>,
    velocity: Vector2<f64>,
    color: [f32; 4],
}

impl Renderable for Circle {
    fn draw(&self, c: Context, gl: &mut GlGraphics) {
        let center = self.center;
        let square = rectangle::centered_square(center.x, center.y, self.radius);
        
        graphics::ellipse(self.color, square, c.transform, gl);
    }
}

impl Physics for Circle {
    fn area(&self) -> f64 {
        self.radius * self.radius * PI
    }

    fn momemnt_of_inertia(&self) -> f64 {
        PI * f64::powi(self.radius, 4) / 4.0
    }

    fn update_vectors(&mut self, dt: f64) {
        self.center += self.velocity * dt ;
        self.velocity += GRAVITY * dt;        
    }

    fn resolve_border_collision(&mut self, dimensions: Vector2<f64>) {        
        let center = self.center;
        let velocity = self.velocity;
        let radius = self.radius;
        if center.x <= radius {
            self.center.x = radius;
            self.velocity.x = -velocity.x;
        } else if center.x >= (dimensions.x - radius) {
            self.center.x = dimensions.x - radius;
            self.velocity.x = -velocity.x;
        }

        if center.y <= radius {
            self.center.y = radius;
            self.velocity.y = -velocity.y;
        }
        else if center.y >= (dimensions.y - radius) {
            self.center.y = dimensions.y - radius;
            self.velocity.y = -velocity.y;
        }
    }

    fn contains_point(&self, point: Vector2<f64>) -> bool {
        (self.center - point).len() < self.radius
    }
}

impl Circle {
    fn new(center: Vector2<f64>, radius: f64, color: [f32; 4]) -> Self {
        Self { 
            radius: radius, 
            center: center, 
            velocity: Vector2::zero(), 
            color: color, 
        }
    }
}

#[derive(Clone)]
struct ConvexPolygon {
    local_vertices: Vec<Vector2<f64>>,
    center: Vector2<f64>,
    velocity: Vector2<f64>,
    rotation: f64,
    angular_velocity: f64,
    color: [f32; 4],
}

impl Renderable for ConvexPolygon {
    fn draw(&self, c: Context, gl: &mut GlGraphics) {
        let verts: Vec<[f64; 2]> = self.get_vertices().iter().map(|&v| v.into()).collect();

        graphics::polygon(self.color, &verts, c.transform, gl);
    }
}

impl Physics for ConvexPolygon {
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

    fn momemnt_of_inertia(&self) -> f64 {
        let n = self.local_vertices.len();
        let mut I = 0.0; 

        for i in 0..n {
            let p1 = self.local_vertices[i];
            let p2 = self.local_vertices[(i + 1) % n];
            I += p1.cross(p2) * (p1.dot(p1) + p1.dot(p2) + p2.dot(p2));
        }
        
        (I / 12.0).abs()
    }

    fn update_vectors(&mut self, dt: f64) {
        self.center += self.velocity * dt;
        self.velocity += GRAVITY * dt;

        self.rotation += self.angular_velocity * dt;
        self.angular_velocity *= ROTATION_DECAY;
    }

    fn resolve_border_collision(&mut self, dimensions: Vector2<f64>) {
        let mut left_border = dimensions.x / 2.0;
        let mut right_border = dimensions.x / 2.0;
        let mut top_border = dimensions.y / 2.0;
        let mut bottom_border = dimensions.y / 2.0;

        for v in self.get_vertices() {
            if v.x < left_border {
                left_border = v.x;
            } else if v.x > right_border {
                right_border = v.x
            }

            if v.y < top_border {
                top_border = v.y;
            } else if v.y > bottom_border {
                bottom_border = v.y;
            }
        }

        if left_border < 0.0 {
            self.center.x -= left_border;
            self.velocity.x = -self.velocity.x;
        } else if right_border > dimensions.x {
            self.center.x += dimensions.x - right_border;
            self.velocity.x = -self.velocity.x
        }

        if top_border < 0.0 {
            self.center.y -= top_border;
            self.velocity.y = -self.velocity.y;
        } else if bottom_border > dimensions.y {
            self.center.y += dimensions.y - bottom_border;
            self.velocity.y = -self.velocity.y;
        }
    }

    fn contains_point(&self, point: Vector2<f64>) -> bool {
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

impl ConvexPolygon {
    fn new_rectangle(position: Vector2<f64>, width: f64, height: f64, color: [f32; 4]) -> Self {
        let half_width = width / 2.0;
        let half_height = height / 2.0;
        let local_verts = vec![
            Vector2::new(-half_width, -half_height), // Top left
            Vector2::new(half_width, -half_height), // Top right
            Vector2::new(half_width, half_height), // Bottom right
            Vector2::new(-half_width, half_height), // Bottom left
        ];

        Self { 
            local_vertices: local_verts, 
            center: position, 
            velocity: Vector2::zero(), 
            rotation: 0.0, 
            angular_velocity: 0.0, 
            color: color 
        }
    }

    fn new_square(position: Vector2<f64>, size: f64, color: [f32; 4]) -> Self {
        Self::new_rectangle(position, size, size, color)
    }

    fn new_regular_polygon(n_sides: u8, radius: f64, position: Vector2<f64>, color: [f32; 4]) -> Self {
        let mut angle = PI * 270.0 / 180.0; // Starting at 270 degrees
        let angle_increment = (2.0 * PI) / n_sides as f64;
        let mut local_verts = vec![];
        for _ in 0..n_sides {
            let x = radius * f64::cos(angle);
            let y = radius * f64::sin(angle);
            local_verts.push(Vector2::new(x, y));
            angle += angle_increment;
        }

        Self { 
            local_vertices: local_verts, 
            center: position, 
            velocity: Vector2::zero(), 
            rotation: 0.0, 
            angular_velocity: 0.0, 
            color: color,
        }
    }

    fn new(vertices: Vec<Vector2<f64>>, position: Vector2<f64>, color: [f32; 4]) -> Self {
        let center = Self::compute_centroid(&vertices);
        let localized_verts: Vec<Vector2<f64>> = vertices.iter().map(|&v| v - center).collect();

        Self { 
            local_vertices: localized_verts, 
            center: position, 
            velocity: Vector2::zero(), 
            rotation: 0.0, 
            angular_velocity: 0.0, 
            color: color 
        }
    }

    fn get_vertices(&self) -> Vec<Vector2<f64>> {
        self.local_vertices.iter().map(|v| v.rotate(self.rotation) + self.center).collect()
    }

    fn closest_vertex_to(&self, point: Vector2<f64>) -> Vector2<f64> {
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

    fn compute_centroid(vertices: &Vec<Vector2<f64>>) -> Vector2<f64> {
        let mut sum_center: Vector2<f64> = Vector2::zero();
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

pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.
    dimensions: Vector2<f64>,
    cursor: Vector2<f64>,
    bodies: Vec<PhysicalBody>,
    target: Option<Vector2<f64>>,
    projectile: PhysicalBody,
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
                graphics::line(color::RED, 1.0, line, c.transform, gl);
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
        let mut change_velocity = |velocity: Vector2<f64>| {
            match &mut self.bodies[0] {
                PhysicalBody::ConvexPolygon(p) => p.velocity = velocity,
                _ => {}
            }
        };

        if args.state == ButtonState::Press {
            match args.button {
                piston::Button::Keyboard(Key::Up) => change_velocity(Vector2::new(0.0, -150.0)),
                piston::Button::Keyboard(Key::Down) => change_velocity(Vector2::new(0.0, 150.0)),
                piston::Button::Keyboard(Key::Left) => change_velocity(Vector2::new(-150.0, 0.0)),
                piston::Button::Keyboard(Key::Right) => change_velocity(Vector2::new(150.0, 0.0)),

                piston::Button::Keyboard(Key::R) => self.color = color::RED,
                piston::Button::Keyboard(Key::G) => self.color = color::GREEN,
                piston::Button::Keyboard(Key::B) => self.color = color::BLUE,

                piston::Button::Keyboard(Key::D3) => self.projectile = PhysicalBody::ConvexPolygon(
                    ConvexPolygon::new_regular_polygon(3, 10.0, self.cursor, self.color)),
                piston::Button::Keyboard(Key::D4) => self.projectile = PhysicalBody::ConvexPolygon(
                    ConvexPolygon::new_regular_polygon(4, 15.0, self.cursor, self.color)),
                piston::Button::Keyboard(Key::D5) => self.projectile = PhysicalBody::ConvexPolygon(
                    ConvexPolygon::new_regular_polygon(5, 20.0, self.cursor, self.color)),
                piston::Button::Keyboard(Key::D6) => self.projectile = PhysicalBody::ConvexPolygon(
                    ConvexPolygon::new_regular_polygon(6, 25.0, self.cursor, self.color)),
                piston::Button::Keyboard(Key::D7) => self.projectile = PhysicalBody::ConvexPolygon(
                    ConvexPolygon::new_regular_polygon(7, 30.0, self.cursor, self.color)),
                piston::Button::Keyboard(Key::D8) => self.projectile = PhysicalBody::ConvexPolygon(
                    ConvexPolygon::new_regular_polygon(8, 35.0, self.cursor, self.color)),
                piston::Button::Keyboard(Key::D9) => self.projectile = PhysicalBody::ConvexPolygon(
                    ConvexPolygon::new_regular_polygon(9, 40.0, self.cursor, self.color)),
                
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
            self.projectile.add_velocity(velocity);
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
        Vector2::new(-15.0, 15.0),
        Vector2::new(-15.0, 0.0),
        Vector2::new(0.0, -15.0),
        Vector2::new(15.0, 0.0),
        Vector2::new(15.0, 15.0) 
    ];

    let player_body = PhysicalBody::ConvexPolygon(ConvexPolygon::new(
        verts,
        Vector2::new(250.0, 250.0), 
        color::PURPLE,
    ));

    let manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()).unwrap();

    // Create a new game and run it.
    let mut app = App {
        gl: GlGraphics::new(opengl),
        dimensions: Vector2 { x: 500.0, y: 500.0 },
        cursor: Vector2::zero(),
        bodies: vec![player_body],
        target: None,
        projectile: PhysicalBody::Circle(Circle::new(Vector2::zero(), 1.0, color::RED)),
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
