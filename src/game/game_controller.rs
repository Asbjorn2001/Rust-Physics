use crate::game::Game;
use crate::physics::collision::CollisionData;
use crate::user_interface::interfaces::*;
use crate::UIComponent;
use crate::Vector2f;
use crate::utils::helpers::*;
use crate::piston::*;
use crate::color;
use crate::physics::circle::Circle;
use crate::physics::polygon::Polygon;
use crate::physics::rigid_body::RigidBody;

pub struct GameController {
    pub game: Game,
    pub state: Box<dyn GameState>,
    pub cursor_pos: Vector2f<f64>,
}

impl GameController {
    pub fn new(game: Game) -> Self {
        Self { 
            state: Box::new(Playing::from(&game)),
            game: game, 
            cursor_pos: Vector2f::zero(),
        }
    }

    pub fn event(&mut self, e: &Event) {
        if let Some(pos) = e.mouse_cursor_args() {
            self.cursor_pos = pos.into();
        }

        if let Some(new_state) = self.state.update(self.cursor_pos, e, &mut self.game) {
            self.state = new_state;
        }
        
        if let Some(Button::Keyboard(key)) = e.press_args() {
            match key {
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
    }
}