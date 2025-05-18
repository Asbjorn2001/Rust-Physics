use crate::game::Game;
use crate::user_interface::interfaces::Interfaces;
use crate::UIComponent;
use crate::Vector2f;
use crate::utils::helpers::*;
use crate::piston::*;
use crate::color;
use crate::physics::circle::Circle;
use crate::physics::polygon::Polygon;
use crate::physics::rigid_body::RigidBody;

pub enum UIMode {
    Game,
    Settings
}

pub struct GameController {
    pub game: Game,
    pub mode: UIMode,
    pub interfaces: Interfaces,
    pub cursor_pos: Vector2f<f64>,
}

impl GameController {
    pub fn new(game: Game) -> Self {
        Self { 
            game: game, 
            mode: UIMode::Game,
            interfaces: Interfaces::initialize(),
            cursor_pos: Vector2f::zero(),
        }
    }

    pub fn event(&mut self, e: &Event) {
        if let Some(pos) = e.mouse_cursor_args() {
            self.cursor_pos = pos.into();
        }

        match self.mode {
            UIMode::Game => {
                if !self.interfaces.game.update(self.cursor_pos, e, &mut self.game) && self.game.enable_launch {
                    if let Some(Button::Mouse(MouseButton::Left)) = e.press_args() {
                        self.game.target = Some(self.cursor_pos);
                    }
                    if let Some(Button::Mouse(MouseButton::Left)) = e.release_args() {
                        self.launch();
                    }
                }        
            }
            UIMode::Settings => { self.interfaces.settings.update(self.cursor_pos, e, &mut self.game); }
        }
            
        if let Some(Button::Keyboard(key)) = e.press_args() {
            match key {
                Key::S => {
                    match self.mode {
                        UIMode::Game => self.mode = UIMode::Settings,
                        UIMode::Settings => self.mode = UIMode::Game,
                    }
                }
                Key::R => self.game.projectile.shape.set_color(color::RED),
                Key::G => self.game.projectile.shape.set_color(color::GREEN),
                Key::B => self.game.projectile.shape.set_color(color::BLUE),

                Key::D1 => self.game.projectile = RigidBody::from(
                    Circle::new(self.cursor_pos, 15.0, self.game.projectile.shape.get_color())),
                Key::D3 => self.game.projectile = RigidBody::from(
                    Polygon::new_regular_polygon(3, 15.0, self.cursor_pos, self.game.projectile.shape.get_color())),
                Key::D4 => self.game.projectile = RigidBody::from(
                    Polygon::new_regular_polygon(4, 25.0, self.cursor_pos, self.game.projectile.shape.get_color())),
                Key::D5 => self.game.projectile = RigidBody::from(
                    Polygon::new_regular_polygon(5, 20.0, self.cursor_pos, self.game.projectile.shape.get_color())),
                Key::D6 => self.game.projectile = RigidBody::from(
                    Polygon::new_regular_polygon(6, 25.0, self.cursor_pos, self.game.projectile.shape.get_color())),
                Key::D7 => self.game.projectile = RigidBody::from(
                    Polygon::new_regular_polygon(7, 30.0, self.cursor_pos, self.game.projectile.shape.get_color())),
                Key::D8 => self.game.projectile = RigidBody::from(
                    Polygon::new_regular_polygon(8, 35.0, self.cursor_pos, self.game.projectile.shape.get_color())),
                Key::D9 => self.game.projectile = RigidBody::from(
                    Polygon::new_regular_polygon(9, 40.0, self.cursor_pos, self.game.projectile.shape.get_color())),
                _ => {}
            }
        }

        if let Some(args) = e.update_args() {
            self.game.contacts.clear();

            if self.game.enable_launch {
                self.game.projectile.shape.set_center(self.cursor_pos);
                let mut color = self.game.projectile.shape.get_color();
                color[3] = 0.5; // Transparency
                self.game.projectile.shape.set_color(color);
            }
            
            for obj in self.game.bodies.as_mut_slice() {
                obj.update_vectors(args.dt, &self.game.settings.physics);
            }

            for i in 0..self.game.bodies.len() {
                for j in (i+1)..self.game.bodies.len() {
                    let (a, b) = get_pair_mut(&mut self.game.bodies, i, j);
                    if let Some(collision) = a.collide_with(b) {
                        let contacts = a.find_contact_points(b, collision.1);
                        for cp in contacts.clone() {
                            self.game.contacts.push(cp);
                        }
                        a.resolve_collision(b, &collision, contacts);
                    }
                }
            }
        }
    }

    fn launch(&mut self) {
        if let Some(target) = self.game.target {
            let velocity = (target - self.cursor_pos) * 2.0;

            let mut color = self.game.projectile.shape.get_color();
            color[3] = 1.0; // Back to solid
            self.game.projectile.shape.set_color(color);
            self.game.projectile.linear_velocity = velocity;
            self.game.bodies.push(self.game.projectile.clone());
    
            self.game.target = None;
        }
    }
}