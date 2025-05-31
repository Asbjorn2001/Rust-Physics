pub mod game_controller;
pub mod game_view;
pub mod benchmarks;

use std::cell::Ref;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::mem::transmute;
use std::path::Path;
use std::rc::Rc;
use std::time::Instant;

use benchmarks::BenchmarkTests;
use graphics::math::scale;
use graphics::math::translate;
use graphics::math::Matrix2d;
use graphics::rectangle;
use graphics::rectangle::square;
use graphics::Rectangle;
use graphics::Transformed;
use piston::UpdateArgs;
use piston_window::TextureSettings;
use rand::distr::Map;
use rand::seq::SliceRandom;

use crate::physics::material::*;
use crate::physics::shape_type::ShapeType;
use crate::physics::rigid_body::RigidBody;
use crate::physics::string_body::Attachment;
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


const PHYSICS_ITERATIONS: usize = 8;
const MAX_SCALE: f64 = 10.0;
const MIN_SCALE: f64 = 0.1;

pub struct Projectile {
    pub target: Option<Vector2f<f64>>,
    pub body: RigidBody,
    pub scale: f64,
}

pub struct StringStart {
    pub position: Vector2f<f64>,
    pub attachment: Option<Attachment>,
}

pub enum Utility {
    Empty,
    Launch,
    String(Option<StringStart>),
}

pub struct GameSettings {
    pub camera: CameraSettings,
    pub view: ViewSettings,
    pub utility: Utility,
    pub debug_mode: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self { 
            camera: CameraSettings::default(),
            view: ViewSettings::default(),
            utility: Utility::Launch,
            debug_mode: false,
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

pub struct PhysicsData {
    pub gravity: Vector2f<f64>,
    pub air_density: f64,
    pub dt: f64,
}

impl Default for PhysicsData {
    fn default() -> Self {
        PhysicsData { 
            gravity: Vector2f { x: 0.0, y: 250.0 }, 
            air_density: 0.08,
            dt: 1.0 / 120.0, 
        }
    }
}

pub struct ContactDebug {
    pub contact: Vector2f<f64>,
    pub normal: Vector2f<f64>,
}

pub struct Game {
    pub settings: GameSettings,
    pub physics: PhysicsData,
    pub bodies: Vec<Rc<RefCell<RigidBody>>>,
    pub projectile: Projectile,
    pub contacts: Vec<ContactDebug>,
    pub textures: HashMap<MaterialName, Rc<Texture>>,
    pub context: Context,
    pub camera_transform: Matrix2d,
    pub strings: Vec<Rc<RefCell<StringBody>>>,
    pub benchmarks: BenchmarkTests,
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
        let floor = RigidBody::new(floor_shape, CONCRETE, true);
        let floor_ref = Rc::new(RefCell::new(floor.clone()));

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

        let mut string1 = StringBody::new(Vector2f::new(640.0, 320.0), Vector2f::new(640.0, 550.0), 10);

        let head = RigidBody::new(ShapeType::Circle(Circle::new(Vector2f::new(640.0, 280.0), 25.0, color::BLACK)), STEEL, false);
        let head_ref = Rc::new(RefCell::new(head.clone()));
        let head_att = Attachment { obj_ref: head_ref.clone(), rel_pos: head.shape.find_closest_surface_point(string1.joints[0].position).0 - head.shape.get_center()};
        string1.joints[0].attachment = Some(head_att);

        let tail = RigidBody::from(Polygon::new_regular_polygon(5, 25.0, Vector2f::new(640.0, 500.0), color::GRAY));
        let tail_ref = Rc::new(RefCell::new(tail.clone()));
        let tail_att = Attachment { obj_ref: tail_ref.clone(), rel_pos: tail.shape.find_closest_surface_point(string1.joints.last().unwrap().position).0 - tail.shape.get_center() };
        string1.joints.last_mut().unwrap().attachment = Some(tail_att);

        let mut string2 = StringBody::new(Vector2f::new(640.0, 680.0), Vector2f::new(640.0, 1000.0), 20);
        
        let floor_att = Attachment { obj_ref: floor_ref.clone(), rel_pos: floor.shape.find_closest_surface_point(string2.joints[0].position).0 - floor.shape.get_center() };
        string2.joints[0].attachment = Some(floor_att);

        let floor_body = RigidBody::from(Polygon::new_square(Vector2f::new(640.0, 1100.0), 50.0, color::GRAY));
        let floor_body_ref = Rc::new(RefCell::new(floor_body.clone()));
        let floor_body_att = Attachment { obj_ref: floor_body_ref.clone(), rel_pos: floor_body.shape.find_closest_surface_point(string2.joints.last().unwrap().position).0 - floor_body.shape.get_center() };
        string2.joints.last_mut().unwrap().attachment = Some(floor_body_att);

        Self { 
            settings: GameSettings::default(), 
            physics: PhysicsData::default(),
            bodies: vec![floor_ref, floor_body_ref, ramp1, ramp2, triangle, head_ref, tail_ref], 
            projectile: Projectile { 
                target: None, 
                body: RigidBody::from(ShapeType::Circle(Circle::new(Vector2f::zero(), 25.0, color::BLACK))), 
                scale: 1.0 
            },
            contacts: vec![], 
            textures: tex_map,
            context: Context::new(),
            camera_transform: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            strings: vec![Rc::new(RefCell::new(string1)), Rc::new(RefCell::new(string2))],
            benchmarks: BenchmarkTests::default(),
        }
    }
}

impl Game {
    pub fn draw(&self, glyphs: &mut GlyphCache<'static, (), Texture>, c: Context, gl: &mut GlGraphics) {
        graphics::clear(color::WHITE, gl);

        for string in self.strings.as_slice() {
            let string = string.borrow();
            string.draw(self.camera_transform, c, gl);
            if self.settings.debug_mode || self.settings.view.show_velocites {
                for joint in string.joints.as_slice() {
                    let start = joint.position;
                    let end = start + joint.velocity * self.physics.dt;
                    let l = [start.x, start.y, end.x, end.y];
                    graphics::line(color::CYAN, 1.0, l, self.camera_transform, gl);
                }
            }
        }
        
        for obj in self.bodies.as_slice() {
            let obj = obj.borrow();
            obj.draw(self.camera_transform, &self.textures.get(&obj.material.name).unwrap(), c, gl);
            if self.settings.view.show_tiles {
                obj.mesh.draw_tile_outline(self.camera_transform.trans_pos(obj.shape.get_center()).rot_rad(obj.shape.get_rotation()), gl);
            }
            if self.settings.view.show_velocites || self.settings.debug_mode {
                let start = obj.shape.get_center();
                let end = start + obj.linear_velocity * self.physics.dt;
                let l = [start.x, start.y, end.x, end.y];
                graphics::line(color::CYAN, 1.0, l, self.camera_transform, gl);
            }

            if self.settings.debug_mode {
                let mut aabb = obj.shape.get_aabb();
                let center = obj.shape.get_center();
                aabb.top_left -= center;
                aabb.bottom_right -= center;
                aabb.bottom_right *= 2.0;
                let rect = [aabb.top_left.x, aabb.top_left.y, aabb.bottom_right.x, aabb.bottom_right.y];
                Rectangle::new_border(color::BLACK, 1.0)
                    .draw(rect, &c.draw_state, self.camera_transform.trans_pos(center), gl);
            }
        }

        if self.settings.view.show_contact_points || self.settings.debug_mode {
            for cd in self.contacts.as_slice() {
                let square = graphics::rectangle::centered_square(cd.contact.x, cd.contact.y, 5.0);
                graphics::ellipse(color::YELLOW, square, self.camera_transform, gl);
                let cn = cd.contact + cd.normal * 15.0;
                let l = [cd.contact.x, cd.contact.y, cn.x, cn.y];
                graphics::line(color::GREEN, 1.0, l, self.camera_transform, gl);
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

    pub fn update(&mut self, dt: f64) {
        self.benchmarks.updating.start();
        self.contacts.clear();
        self.physics.dt = dt;

        for obj in self.bodies.as_slice() {
            let mut obj = obj.borrow_mut();
            obj.update_velocity(&self.physics);
        }

        // Detect body on body collisions and push out
        self.benchmarks.rigid_collision_detection.start();
        let mut collisions = vec![];
        for i in 0..self.bodies.len() {
            for j in (i+1)..self.bodies.len() {
                let (a, b) = get_pair_mut(&mut self.bodies, i, j);
                let mut a = a.borrow_mut();
                let mut b = b.borrow_mut();
                if let Some(collision) = a.collide_with(&mut b, self.physics.dt) {
                    self.contacts.extend(collision.contacts.iter().map(|&contact| ContactDebug { contact, normal: collision.normal}));
                    collisions.push((i, j, collision));
                }
            }
        }
        self.benchmarks.rigid_collision_detection.stop(Some(self.bodies.len()));

        // Resolve collisions
        self.benchmarks.rigid_collision_solving.start();
        let mut rng = rand::rng();
        for _ in 0..PHYSICS_ITERATIONS {
            collisions.shuffle(&mut rng);
            for (i, j, collision) in collisions.as_slice() {
                let (a, b) = get_pair_mut(&mut self.bodies, *i, *j);
                let mut a = a.borrow_mut();
                let mut b = b.borrow_mut();
                a.resolve_collision(&mut b, collision);
            }
        }    
        self.benchmarks.rigid_collision_solving.stop(Some(collisions.len()));

        // Resolve constraints with verlet integration
        let mut new_strings = vec![];
        for string in self.strings.as_slice() {
            let mut string = string.borrow_mut();
            if let Some(new_string) = string.resolve_constraints(
                &self.physics, 
                &self.bodies, 
                &mut self.contacts, 
            ) {
                new_strings.push(Rc::new(RefCell::new(new_string)));
            }
        }
        self.strings.extend(new_strings);  

        for obj_ref in self.bodies.as_mut_slice() {
            obj_ref.borrow_mut().update_position(self.physics.dt);
        }
        self.benchmarks.updating.stop(None);
    }
}