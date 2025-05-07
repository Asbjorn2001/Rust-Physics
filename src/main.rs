extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use glutin_window::GlutinWindow as Window;
use graphics::math::{separation, translate, Vec2d};
use graphics::*;
use opengl_graphics::{GlGraphics, OpenGL, GLSL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::{self, WindowSettings};
use piston::{Button, ButtonArgs, ButtonEvent, ButtonState, Key, Size};
use rand::rand_core::le;
use core::f64;
use std::f64::consts::PI;

pub mod utils;
use utils::*;

const EPSILON: f64 = 0.0005;
const GRAVITY: Vector2<f64> = Vector2 { x: 0.0, y: 9.816 };


trait Renderable {
    fn draw(&self, c: Context, gl: &mut GlGraphics);
}

trait Physics : Renderable {
    fn get_area(&self) -> f64;

    fn update_vectors(&mut self, dt: f64);

    fn resolve_border_collision(&mut self, dimensions: Vector2<f64>);
}

enum PhysicalBody {
    Circle(Circle),
    Rectangle(Rectangle),
    ConvexPolygon(ConvexPolygon)
}

impl Renderable for PhysicalBody {
    fn draw(&self, c: Context, gl: &mut GlGraphics) {
        match self {
            PhysicalBody::Circle(circle) => circle.draw(c, gl),
            PhysicalBody::Rectangle(rect) => rect.draw(c, gl),
            PhysicalBody::ConvexPolygon(poly) => poly.draw(c, gl),
        }
    }
}

impl Physics for PhysicalBody {
    fn get_area(&self) -> f64 {
        match  self {
            PhysicalBody::Circle(c) => c.get_area(),
            PhysicalBody::Rectangle(r) => r.get_area(),
            PhysicalBody::ConvexPolygon(p) => p.get_area(),
        }
    }   

    fn update_vectors(&mut self, dt: f64) {
        match self {
            PhysicalBody::Circle(c) => c.update_vectors(dt),
            PhysicalBody::Rectangle(r) => r.update_vectors(dt),
            PhysicalBody::ConvexPolygon(p) => p.update_vectors(dt),
        }
    }

    fn resolve_border_collision(&mut self, dimensions: Vector2<f64>) {
        match self {
            PhysicalBody::Circle(c) => c.resolve_border_collision(dimensions),
            PhysicalBody::Rectangle(r) => r.resolve_border_collision(dimensions),
            PhysicalBody::ConvexPolygon(p) => p.resolve_border_collision(dimensions),
        }
    }
}

impl PhysicalBody {
    fn resolve_collision_with(&mut self, other: &mut PhysicalBody) {
        match (self, other) {
            (PhysicalBody::Circle(ca), PhysicalBody::Circle(cb)) => resolve_circle_circle(ca, cb),
            (PhysicalBody::ConvexPolygon(pa), PhysicalBody::ConvexPolygon(pb)) => resolve_poly_poly(pa, pb),
            (PhysicalBody::Circle(c), PhysicalBody::ConvexPolygon(p)) |
            (PhysicalBody::ConvexPolygon(p), PhysicalBody::Circle(c)) => resolve_circle_poly(c, p),
            _ => {}
        }
    }
}

fn resolve_circle_circle(a: &mut Circle, b: &mut Circle) {
    let delta_dist = a.center.next - b.center.next;
    let sum_radius = a.radius + b.radius;
    if delta_dist.len() < sum_radius {
        let m1 = a.get_area();
        let m2 = b.get_area();
        let v1 = a.velocity.curr;
        let v2 = b.velocity.curr;
        
        // Push the objects away from each other
        let a_next = b.center.next + delta_dist.normalize() * sum_radius;
        b.center.next = a.center.next - delta_dist.normalize() * sum_radius;
        a.center.next = a_next;

        // Update velocites
        a.velocity.next = v1 - delta_dist * (2.0 * m2) / (m1 + m2) * (v1 - v2).dot(delta_dist) / f64::powi(delta_dist.len(), 2);
        b.velocity.next = v2 - delta_dist * (2.0 * m1) / (m1 + m2) * (v2 - v1).dot(delta_dist) / f64::powi(delta_dist.len(), 2);
    }
}

fn resolve_poly_poly(a: &mut ConvexPolygon, b: &mut ConvexPolygon) {
    let a_result= a.find_min_seperation(b);
    let b_result = b.find_min_seperation(a);
    let (sep, mut col_normal, pen_vert) = if a_result.0 > b_result.0 { a_result } else { b_result };
    if sep <= 0.0 {
        if col_normal.dot(b.centroid.next - a.centroid.next) < 0.0 {
            col_normal = -col_normal;
        }

        let relative_velocity = a.velocity.curr - b.velocity.curr;
        let impulse = -1.5 * relative_velocity.dot(col_normal) / (1.0 / a.get_area() + 1.0 / b.get_area());

        a.centroid.next += col_normal * sep / 2.0;
        b.centroid.next -= col_normal * sep / 2.0;

        a.velocity.next += col_normal * impulse / a.get_area();
        b.velocity.next -= col_normal * impulse / b.get_area();

        //let moments_of_inertia = (a.dims.x * a.dims.x + b.dims.y * b.dims.y) / 12.0;
        let contact_point = pen_vert + col_normal * sep / 2.0;

        //a.angular_velocity.next += (contact_point - a.center.next).cross(col_normal * impulse) / (a.get_area() * moments_of_inertia);
        //b.angular_velocity.next += (contact_point - b.center.next).cross(-col_normal * impulse) / (b.get_area() * moments_of_inertia);
    } else {
        a.color = color::BLACK;
        b.color = color::BLACK;
    }
}

fn resolve_circle_poly(c: &mut Circle, p: &mut ConvexPolygon) {

}

#[derive(Clone, Copy)]
struct Circle {
    radius: f64,
    center: Delta<Vector2<f64>>,
    velocity: Delta<Vector2<f64>>,
    color: [f32; 4],
}

impl Renderable for Circle {
    fn draw(&self, c: Context, gl: &mut GlGraphics) {
        let center = self.center.curr;
        let square = rectangle::centered_square(center.x, center.y, self.radius);
        
        graphics::ellipse(self.color, square, c.transform, gl);
    }
}

impl Physics for Circle {
    fn get_area(&self) -> f64 {
        self.radius * self.radius * PI
    }

    fn update_vectors(&mut self, dt: f64) {
        self.velocity.curr = self.velocity.next;
        self.velocity.next += GRAVITY * dt;

        self.center.curr = self.center.next;
        self.center.next += (self.velocity.curr + self.velocity.next) / 2.0 * dt ;
    }

    fn resolve_border_collision(&mut self, dimensions: Vector2<f64>) {        
        let center = self.center.next;
        let velocity = self.velocity.next;
        let radius = self.radius;
        if center.x <= radius {
            self.center.next.x = radius;
            self.velocity.next.x = -velocity.x;
        } else if center.x >= (dimensions.x - radius) {
            self.center.next.x = dimensions.x - radius;
            self.velocity.next.x = -velocity.x;
        }

        if center.y <= radius {
            self.center.next.y = radius;
            self.velocity.next.y = -velocity.y;
        }
        else if center.y >= (dimensions.y - radius) {
            self.center.next.y = dimensions.y - radius;
            self.velocity.next.y = -velocity.y;
        }
    }
}

#[derive(Clone, Copy)]
struct Rectangle {
    dims: Vector2<f64>,
    center: Delta<Vector2<f64>>,
    velocity: Delta<Vector2<f64>>,
    rotation: Delta<f64>, // radians
    angular_velocity: Delta<f64>,
    color: [f32; 4],
}

impl Renderable for Rectangle {
    fn draw(&self, c: Context, gl: &mut GlGraphics) {
        let transform = c.transform
                                        .trans_pos(self.center.curr)
                                        .rot_rad(self.rotation.curr);

        let half_size = self.dims / 2.0;
        graphics::rectangle_from_to(self.color, -half_size, half_size, transform, gl);
    }
}

impl Physics for Rectangle {
    fn get_area(&self) -> f64 {
        (self.dims.x * self.dims.y).abs()
    }

    fn update_vectors(&mut self, dt: f64) {
        self.velocity.curr = self.velocity.next;
        self.velocity.next += GRAVITY * dt;

        self.center.curr = self.center.next;
        self.center.next += (self.velocity.curr + self.velocity.next) / 2.0 * dt;

        self.angular_velocity.curr = self.angular_velocity.next;
        self.angular_velocity.next *= 0.99;

        self.rotation.curr = self.rotation.next;
        self.rotation.next += (self.angular_velocity.curr + self.angular_velocity.next) / 2.0 * dt;
    }

    fn resolve_border_collision(&mut self, dimensions: Vector2<f64>) {
        let half_size = self.dims / 2.0;
        let top_left = self.center.next - half_size;
        let bottom_right = top_left + self.dims;
        let velocity = self.velocity.next;
        if top_left.x <= 0.0 {
            self.center.next.x = half_size.x;
            self.velocity.next.x = -velocity.x
        } else if bottom_right.x >= dimensions.x {
            self.center.next.x = dimensions.x - half_size.x;
            self.velocity.next.x = -velocity.x;
        }

        if top_left.y <= 0.0 {
            self.center.next.y = half_size.y;
            self.velocity.next.y = -velocity.y;
        } else if bottom_right.y >= dimensions.y {
            self.center.next.y = dimensions.y - half_size.y;
            self.velocity.next.y = -velocity.y;
        }
    }
}

struct ConvexPolygon {
    local_vertices: Vec<Vector2<f64>>,
    centroid: Delta<Vector2<f64>>,
    velocity: Delta<Vector2<f64>>,
    rotation: Delta<f64>,
    anuglar_velocity: Delta<f64>,
    color: [f32; 4],
}

impl Renderable for ConvexPolygon {
    fn draw(&self, c: Context, gl: &mut GlGraphics) {
        let verts: Vec<[f64; 2]> = self.get_vertices().iter().map(|&v| v.into()).collect();
        let dims = c.get_view_size();

        graphics::polygon(self.color, &verts, c.transform.trans(-dims[0] / 2.0, -dims[1] / 2.0), gl);
    }
}

impl Physics for ConvexPolygon {
    fn get_area(&self) -> f64 {
        let len = self.local_vertices.len();
        let mut sum = 0.0;
        for i in 0..len {
            let (p1, p2) = if i == len - 1 {
                (self.local_vertices[len - 1], self.local_vertices[0])
            } else {
                (self.local_vertices[i], self.local_vertices[i + 1])
            };
            sum += p1.cross(p2);
        }
        sum.abs() / 2.0
    }

    fn update_vectors(&mut self, dt: f64) {
        self.velocity.curr = self.velocity.next;
        self.velocity.next += GRAVITY * dt;

        self.centroid.curr = self.centroid.next;
        self.centroid.next += (self.velocity.curr + self.velocity.next) / 2.0 * dt;

        self.anuglar_velocity.curr = self.anuglar_velocity.next;
        self.anuglar_velocity.next *= 0.99;

        self.rotation.curr = self.rotation.next;
        self.rotation.next += (self.anuglar_velocity.curr + self.anuglar_velocity.next) / 2.0 * dt;
    }

    fn resolve_border_collision(&mut self, dimensions: Vector2<f64>) {
        
    }
}

impl ConvexPolygon {
    fn new_rectangle(position: Vector2<f64>, width: f64, height: f64, color: [f32; 4]) -> Self {
        let w = width / 2.0;
        let h = height / 2.0;
        let verts = vec![
            position + Vector2::new(-w, -h), // Top left
            position + Vector2::new(w, -h), // Top right
            position + Vector2::new(w, h), // Bottom right
            position + Vector2::new(-w, h), // Bottom left
        ];

        Self { 
            local_vertices: verts, 
            centroid: Delta::new(position), 
            velocity: Delta::new(Vector2::zero()), 
            rotation: Delta::new(0.0), 
            anuglar_velocity: Delta::new(0.0), 
            color: color 
        }
    }

    fn new_square(position: Vector2<f64>, size: f64, color: [f32; 4]) -> Self {
        Self::new_rectangle(position, size, size, color)
    }

    fn new(vertices: Vec<Vector2<f64>>, position: Vector2<f64>, color: [f32; 4]) -> Self {
        let initial_centroid = Self::compute_centroid(&vertices);
        let localized_verts: Vec<Vector2<f64>> = vertices.iter().map(|&v| v - initial_centroid).collect();

        Self { 
            local_vertices: vertices, 
            centroid: Delta::new(position), 
            velocity: Delta::new(Vector2::zero()), 
            rotation: Delta::new(0.0), 
            anuglar_velocity: Delta::new(0.0), 
            color: color 
        }
    }

    fn compute_centroid(vertices: &Vec<Vector2<f64>>) -> Vector2<f64> {
        let mut sum_center: Vector2<f64> = Vector2::zero();
        let mut sum_weight = 0.0;
        let len = vertices.len();
        for i in 0..len {
            let (prev, curr, next) = 
            match i {
                i if i == 0 => (vertices[len - 1], vertices[i], vertices[i + 1]),
                i if i == len - 1 => (vertices[i - 1], vertices[i], vertices[0]),
                i => (vertices[i - 1], vertices[i], vertices[i + 1]),
            };
            let weight = (curr - next).len() + (curr - prev).len();
            sum_center += curr * weight;
            sum_weight += weight;
        }
        sum_center / sum_weight
    }

    fn get_vertices(&self) -> Vec<Vector2<f64>> {
        self.local_vertices.iter().map(|v| v.rotate(self.rotation.next) + self.centroid.next).collect()
    }

    fn find_min_seperation(&self, other: &ConvexPolygon) -> (f64, Vector2<f64>, Vector2<f64>) {
        let mut seperation = (f64::NEG_INFINITY, Vector2::new(0.0, 0.0), Vector2::new(0.0, 0.0));
        let perpendicular = |vertices: &Vec<Vector2<f64>>, i| -> Vector2<f64> {
            let last_index = vertices.len() - 1;
            let normal: Vector2<f64>;
            match i {
                i if i < last_index => normal = vertices[i] - vertices[i + 1],
                i if i == last_index => normal = vertices[last_index] - vertices[0],
                _ => normal = Vector2::new(0.0, 0.0)
            }
            normal.perpendicular()
        };

        let verts_a = self.get_vertices();
        let mut pen_vertex = verts_a[0];
        for i in 0..verts_a.len() {
            let normal = perpendicular(&verts_a, i).normalize();
            let mut min_sep = f64::INFINITY;

            for vb in other.get_vertices() {
                let sep = (vb - verts_a[i]).dot(normal);
                if sep < min_sep {
                    pen_vertex = vb;
                    min_sep = sep;
                }
            }

            if min_sep > seperation.0 {
                seperation = (min_sep, normal, pen_vertex);
            }
        }
        seperation
    }
}

pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.
    dimensions: Vector2<f64>,
    bodies: Vec<PhysicalBody>,
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
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        for obj in self.bodies.as_mut_slice() {
            obj.update_vectors(args.dt);
        }

        for i in 0..self.bodies.len() {
            for j in (i+1)..self.bodies.len() {
                let (a, b) = get_pair_mut(&mut self.bodies, i, j);
                a.resolve_collision_with(b);
            }
        }

        for body in self.bodies.as_mut_slice() {
            body.resolve_border_collision(self.dimensions);
        }
    }

    fn handle_button_events(&mut self, args: &ButtonArgs) {
        if args.state == ButtonState::Press {
            let create_circle = |col: [f32; 4]| -> PhysicalBody {
                PhysicalBody::Circle(Circle { 
                    radius: 5.0,    
                    center: Delta::new(self.dimensions * 0.5), 
                    velocity: Delta::new(Vector2::random_direction() * 30.0),
                    color: col
                })
            };

            let create_rect = |col: [f32; 4]| -> PhysicalBody {
                PhysicalBody::ConvexPolygon(ConvexPolygon::new_rectangle(self.dimensions * 0.5, 30.0, 15.0, col)
            )};

            let mut change_speed = |multiplier: f64| {
                for obj in self.bodies.as_mut_slice() {
                    //obj.next_velocity *= multiplier;
                }
            };

            match args.button {
                piston::Button::Keyboard(Key::R) => self.bodies.push(create_circle(color::RED)),
                piston::Button::Keyboard(Key::G) => self.bodies.push(create_circle(color::GREEN)),
                piston::Button::Keyboard(Key::B) => self.bodies.push(create_circle(color::BLUE)),
                piston::Button::Keyboard(Key::Up) => change_speed(1.1),
                piston::Button::Keyboard(Key::Down) => change_speed(0.9),
                piston::Button::Keyboard(Key::S) => self.bodies.push(create_rect(color::BLACK)),
                _ => ()
            }
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

    let starting_bodies: Vec<PhysicalBody> = 
    vec![PhysicalBody::Circle(Circle {
        radius: 20.0, 
        center: Delta::new(Vector2::new(50.0, 100.0)), 
        velocity: Delta::new(Vector2::new(100.0, -30.0)), 
        color: color::BLACK
    })];

    // Create a new game and run it.
    let mut app = App {
        gl: GlGraphics::new(opengl),
        bodies: starting_bodies,
        dimensions: Vector2 { x: 500.0 / 2.0, y: 500.0 / 2.0 },
    };

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            app.render(&args);
        }

        if let Some(args) = e.update_args() {
            app.update(&args);
        }

        if let Some(args) = e.button_args() {
            app.handle_button_events(&args);
        }
    }
}
