pub mod game_controller;
pub mod game_view;

use std::cell::Ref;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::mem::transmute;
use std::path::Path;
use std::rc::Rc;

use graphics::math::scale;
use graphics::math::translate;
use graphics::math::Matrix2d;
use graphics::Transformed;
use piston::UpdateArgs;
use piston_window::TextureSettings;
use rand::distr::Map;
use rand::seq::SliceRandom;

use crate::physics::material::*;
use crate::physics::shape_type::ShapeType;
use crate::physics::rigid_body::RigidBody;
use crate::Vector2f;
use crate::utils::helpers::*;
use crate::physics::collision::CollisionData;
use crate::GlGraphics;
use crate::GlyphCache;
use crate::Texture;
use crate::physics::circle::Circle;
use crate::physics::polygon::Polygon;
use crate::physics::shape::Renderable;
use crate::color;
use crate::physics::rigid_body::*;
use crate::physics::string_body::StringBody;
use crate::Context;


const PHYSICS_ITERATIONS: usize = 10;
const MAX_SCALE: f64 = 10.0;
const MIN_SCALE: f64 = 0.1;

pub struct GameSettings {
    pub camera: CameraSettings,
    pub view: ViewSettings,
    pub physics: PhysicsSettings,
    pub enable_launch: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self { 
            camera: CameraSettings::default(),
            view: ViewSettings::default(),
            physics: PhysicsSettings::default(), 
            enable_launch: true,
        }
    }
}

pub struct CameraSettings {
    pub scale: f64,
    pub position: Vector2f<f64>,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            scale: 1.0,
            position: Vector2f::new(640.0, 360.0),
        }
    }
}

pub struct ViewSettings {
    pub show_velocites: bool,
    pub show_contact_points: bool,
    pub show_tiles: bool,
}

impl Default for ViewSettings {
    fn default() -> Self {
        Self { 
            show_velocites: false, 
            show_contact_points: false, 
            show_tiles: false,
        }
    }
}

pub struct PhysicsSettings {
    pub gravity: Vector2f<f64>,
    pub air_density: f64,
}

impl Default for PhysicsSettings {
    fn default() -> Self {
        PhysicsSettings { gravity: Vector2f { x: 0.0, y: 250.0 }, air_density: 0.08 }
    }
}

pub struct Game {
    pub settings: GameSettings,
    pub bodies: Vec<Rc<RefCell<RigidBody>>>,
    pub target: Option<Vector2f<f64>>,
    pub projectile: RigidBody,
    pub projectile_scale: f64,
    pub contacts: Vec<Vector2f<f64>>,
    pub textures: HashMap<MaterialName, Rc<Texture>>,
    pub context: Context,
    pub camera_transform: Matrix2d,
    pub strings: Vec<Rc<RefCell<StringBody>>>,
}

impl Default for Game {
    fn default() -> Self {
        // Create bodies
        let floor_shape = ShapeType::Polygon(Polygon::new_rectangle(
            Vector2f::new(640.0, 650.0), 
            1000.0, 
            50.0, 
            color::OLIVE
        ));
        let floor = Rc::new(RefCell::new(RigidBody::new(floor_shape, CONCRETE, true)));

        let mut ramp_shape1 = ShapeType::Polygon(Polygon::new_rectangle(
            Vector2f::new(450.0, 300.0), 
            400.0,
            25.0, 
            color::TEAL
        ));
        let mut ramp_shape2 = ramp_shape1.clone();

        ramp_shape1.rotate(0.5);

        ramp_shape2.translate(Vector2f::new(400.0, -150.0));
        ramp_shape2.rotate(-0.5);
        ramp_shape2.set_color(color::MAROON);

        let ramp1 = Rc::new(RefCell::new(RigidBody::new(ramp_shape1, STEEL, true)));
        let ramp2 = Rc::new(RefCell::new(RigidBody::new(ramp_shape2, ICE,  true)));

        let triangle = Rc::new(RefCell::new(RigidBody::new(
            ShapeType::Polygon(
                Polygon::new_regular_polygon(3, 60.0, Vector2f::new(800.0, 595.0), color::GREEN)),
                WOOD,
                true,
        )));

        let tex_settings = TextureSettings::new();
        let tex_path = Path::new("./src/assets/textures/materials");        
        let mut tex_map = HashMap::new();

        tex_map.insert(MaterialName::Concrete, Rc::new(Texture::from_path(Path::new(&tex_path).join("concrete.png"), &tex_settings).unwrap()));
        tex_map.insert(MaterialName::Steel, Rc::new(Texture::from_path(Path::new(&tex_path).join("steel.png"), &tex_settings).unwrap()));
        tex_map.insert(MaterialName::Ice, Rc::new(Texture::from_path(Path::new(&tex_path).join("ice.png"), &tex_settings).unwrap()));
        tex_map.insert(MaterialName::Wood, Rc::new(Texture::from_path(Path::new(&tex_path).join("wood.png"), &tex_settings).unwrap()));


        let head = RigidBody::new(ShapeType::Circle(Circle::new(Vector2f::new(640.0, 300.0), 25.0, color::BLACK)), STEEL, false);
        let tail = RigidBody::from(Polygon::new_regular_polygon(5, 25.0, Vector2f::new(640.0, 500.0), color::GRAY));
        let head = Rc::new(RefCell::new(head));
        let tail = Rc::new(RefCell::new(tail));

        let mut string = StringBody::new(Vector2f::new(640.0, 300.0), 20);
        string.joints[0].attachment = Some(head.clone());
        string.joints.last_mut().unwrap().attachment = Some(tail.clone());

        Self { 
            settings: GameSettings::default(), 
            bodies: vec![floor, ramp1, ramp2, triangle, head, tail], 
            target: None, 
            projectile: RigidBody::from(ShapeType::Circle(Circle::new(Vector2f::zero(), 25.0, color::BLACK))), 
            projectile_scale: 1.0, 
            contacts: vec![], 
            textures: tex_map,
            context: Context::new(),
            camera_transform: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            strings: vec![Rc::new(RefCell::new(string))],
        }
    }
}

impl Game {
    pub fn draw(&self, glyphs: &mut GlyphCache<'static, (), Texture>, c: Context, gl: &mut GlGraphics) {
        graphics::clear(color::WHITE, gl);

        for string in self.strings.as_slice() {
            string.borrow().draw(self.camera_transform, c, gl);
        }
        for obj in self.bodies.as_slice() {
            let obj = obj.borrow();
            obj.draw(self.camera_transform, &self.textures.get(&obj.material.name).unwrap(), c, gl);
            if self.settings.view.show_tiles {
                obj.mesh.draw_tile_outline(self.camera_transform.trans_pos(obj.shape.get_center()).rot_rad(obj.shape.get_rotation()), gl);
            }
            if self.settings.view.show_velocites {
                let o = obj.shape.get_center();
                let vel = obj.linear_velocity;
                let l = [o.x, o.y, o.x + vel.x, o.y + vel.y];
                graphics::line(color::CYAN, 1.0, l, self.camera_transform, gl);
            }
        }

        if self.settings.view.show_contact_points {
            for cp in self.contacts.as_slice() {
                let square = graphics::rectangle::centered_square(cp.x, cp.y, 5.0);
                graphics::ellipse(color::YELLOW, square, self.camera_transform, gl);
            }
        }
    }

    pub fn update_camera(&mut self, c: Context) {
        self.context = c;
        let dims = Vector2f::from(c.get_view_size());
        self.camera_transform = c.transform.trans_pos(dims / 2.0)
                                .scale(self.settings.camera.scale, self.settings.camera.scale)
                                .trans_pos(-self.settings.camera.position);
    }

    pub fn update(&mut self, args: &UpdateArgs) {
        self.contacts.clear();
        
        for obj in self.bodies.as_mut_slice() {
            let mut obj = obj.borrow_mut();
            obj.update_vectors(args.dt, &self.settings.physics);
        }

        // Detect collisions
        let mut collisions = vec![];
        for i in 0..self.bodies.len() {
            for j in (i+1)..self.bodies.len() {
                let (a, b) = get_pair_mut(&mut self.bodies, i, j);
                let mut a = a.borrow_mut();
                let mut b = b.borrow_mut();
                if let Some(CollisionData(_, normal)) = a.collide_with(&mut b) {
                    let contacts = a.find_contact_points(&mut b, normal);
                    self.contacts.extend(contacts.clone());
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
                let mut a = a.borrow_mut();
                let mut b = b.borrow_mut();
                a.resolve_collision(&mut b, normal, contacts);
            }
        }        

        // Resolve constraints
        let mut new_strings = vec![];
        for string in self.strings.as_slice() {
            let mut string = string.borrow_mut();
            if let Some(new_string) = string.update(args.dt, &self.settings.physics, &self.bodies) {
                new_strings.push(Rc::new(RefCell::new(new_string)));
            }
        }
        self.strings.extend(new_strings);
    }
}