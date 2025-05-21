pub mod game_controller;
pub mod game_view;

use piston::UpdateArgs;

use crate::RigidBody;
use crate::Vector2f;
use crate::utils::helpers::*;
use crate::physics::collision::CollisionData;
use crate::GlGraphics;
use crate::GlyphCache;
use crate::Texture;
use crate::user_interface::interfaces::*;
use crate::graphics::*;
use crate::physics::shape::Renderable;
use crate::piston::*;
use crate::color;


const PHYSICS_ITERATIONS: usize = 10;

pub struct GameSettings {
    pub physics: PhysicsSettings,
    pub view: ViewSettings,
    pub enable_launch: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self { 
            physics: PhysicsSettings::default(), 
            view: ViewSettings::default(),
            enable_launch: true,
        }
    }
}

pub struct ViewSettings {
    pub show_velocites: bool,
    pub show_contact_points: bool,
}

impl Default for ViewSettings {
    fn default() -> Self {
        Self { 
            show_velocites: true, 
            show_contact_points: true 
        }
    }
}

pub struct PhysicsSettings {
    pub gravity: Vector2f<f64>,
    pub air_density: f64,
}

impl Default for PhysicsSettings {
    fn default() -> Self {
        PhysicsSettings { gravity: Vector2f { x: 0.0, y: 100.0 }, air_density: 0.08 }
    }
}

pub struct Game {
    pub settings: GameSettings,
    pub bodies: Vec<RigidBody>,
    pub target: Option<Vector2f<f64>>,
    pub projectile: RigidBody,
    pub contacts: Vec<Vector2f<f64>>,
}

impl Game {
    pub fn draw(&self, glyphs: &mut GlyphCache<'static, (), Texture>, c: Context, gl: &mut GlGraphics) {
        graphics::clear(color::WHITE, gl);

        for obj in self.bodies.as_slice() {
            obj.shape.draw(c, gl);
            if self.settings.view.show_velocites {
                let o = obj.shape.get_center();
                let vel = obj.linear_velocity;
                let line = [o.x, o.y, o.x + vel.x, o.y + vel.y];
                graphics::line(color::CYAN, 1.0, line, c.transform, gl);
            }
        }

        if self.settings.view.show_contact_points {
            for cp in self.contacts.as_slice() {
                let square = graphics::rectangle::centered_square(cp.x, cp.y, 5.0);
                graphics::ellipse(color::YELLOW, square, c.transform, gl);
            }
        }
    }

    pub fn update(&mut self, args: &UpdateArgs) {
        self.contacts.clear();
        
        for obj in self.bodies.as_mut_slice() {
            obj.update_vectors(args.dt, &self.settings.physics);
        }

        // Detect collisions
        let mut collisions = vec![];
        for i in 0..self.bodies.len() {
            for j in (i+1)..self.bodies.len() {
                let (a, b) = get_pair_mut(&mut self.bodies, i, j);
                if let Some(CollisionData(_, normal)) = a.collide_with(b) {
                    let contacts = a.find_contact_points(b, normal);
                    for cp in contacts.clone() {
                        self.contacts.push(cp);
                    }
                    collisions.push((i, j, normal, contacts));
                }
            }
        }

        // Resolve collisions
        for _ in 0..PHYSICS_ITERATIONS {
            for (i, j, normal, contacts) in collisions.as_slice() {
                let (a, b) = get_pair_mut(&mut self.bodies, *i, *j);
                a.resolve_collision(b, normal, contacts);
            }
        }
    }
}