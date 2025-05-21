pub mod game_controller;
pub mod game_view;

use piston::UpdateArgs;
use rand::seq::SliceRandom;

use crate::physics::shape_type::ShapeType;
use crate::physics::rigid_body::RigidBody;
use crate::Vector2f;
use crate::utils::helpers::*;
use crate::physics::collision::CollisionData;
use crate::GlGraphics;
use crate::GlyphCache;
use crate::Texture;
use crate::Circle;
use crate::Polygon;
use crate::graphics::*;
use crate::physics::shape::Renderable;
use crate::color;
use crate::physics::rigid_body::*;


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
    pub projectile: ShapeType,
    pub projectile_scale: f64,
    pub contacts: Vec<Vector2f<f64>>,
}

impl Default for Game {
    fn default() -> Self {
        // Create bodies
        let floor_shape = ShapeType::Polygon(Polygon::new_rectangle(
            Vector2f::new(640.0, 650.0), 
            800.0, 
            50.0, 
            color::OLIVE
        ));
        let floor = RigidBody::new(floor_shape, 1.0, BASE_ELASTICITY, BASE_STATIC_FRICTION, BASE_DYNAMIC_FRICTION, true);

        let mut ramp_shape1 = ShapeType::Polygon(Polygon::new_rectangle(
            Vector2f::new(450.0, 400.0), 
            400.0, 
            25.0, 
            color::TEAL
        ));
        let mut ramp_shape2 = ramp_shape1.clone();

        ramp_shape1.rotate(0.5);

        ramp_shape2.translate(Vector2f::new(400.0, -200.0));
        ramp_shape2.rotate(-0.5);
        ramp_shape2.set_color(color::MAROON);

        let ramp1 = RigidBody::new(ramp_shape1, 1.0, BASE_ELASTICITY, BASE_STATIC_FRICTION, BASE_DYNAMIC_FRICTION, true); 
        let ramp2 = RigidBody::new(ramp_shape2, 1.0, BASE_ELASTICITY, BASE_STATIC_FRICTION, BASE_DYNAMIC_FRICTION, true);
        Self { 
            settings: GameSettings::default(), 
            bodies: vec![floor, ramp1, ramp2], 
            target: None, 
            projectile: ShapeType::Circle(Circle::new(Vector2f::zero(), 25.0, color::RED)), 
            projectile_scale: 1.0, 
            contacts: vec![], 
        }
    }
}

impl Game {
    pub fn draw(&self, glyphs: &mut GlyphCache<'static, (), Texture>, c: Context, gl: &mut GlGraphics) {
        graphics::clear(color::WHITE, gl);

        for obj in self.bodies.as_slice() {
            obj.shape.draw(c.transform, gl);
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
        let mut rng = rand::rng();
        for _ in 0..PHYSICS_ITERATIONS {
            collisions.shuffle(&mut rng);
            for (i, j, normal, contacts) in collisions.as_slice() {
                let (a, b) = get_pair_mut(&mut self.bodies, *i, *j);
                a.resolve_collision(b, normal, contacts);
            }
        }
    }
}