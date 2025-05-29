use crate::game::Game;
use crate::game_state::*;
use crate::Vector2f;
use piston_window::*;

use super::MAX_SCALE;
use super::MIN_SCALE;

pub struct GameController {
    pub game: Game,
    pub state: Box<dyn GameState>,
    pub cursor_pos: Vector2f<f64>,
}

impl GameController {
    pub fn new(game: Game) -> Self {
        Self { 
            state: Box::new(main_state::MainState::from(&game)),
            game: game, 
            cursor_pos: Vector2f::zero(),
        }
    }

    pub fn event(&mut self, e: &Event) {
        if let Some(pos) = e.mouse_cursor_args() {
            self.cursor_pos = pos.into();
        }

        if !self.game.settings.debug_mode {
            if let Some(new_state) = self.state.update(self.cursor_pos, e, &mut self.game) {
                self.state = new_state;
            }
        }
        

        if let Some(Button::Keyboard(key)) = e.press_args() {
            match key {
                Key::W => self.game.settings.camera.scale = MAX_SCALE.min(self.game.settings.camera.scale * 1.5),
                Key::S => self.game.settings.camera.scale = MIN_SCALE.max(self.game.settings.camera.scale * 0.75),
                Key::Up => self.game.settings.camera.position.y -= 50.0,
                Key::Down => self.game.settings.camera.position.y += 50.0,
                Key::Left => self.game.settings.camera.position.x -= 50.0,
                Key::Right => self.game.settings.camera.position.x += 50.0,
                Key::Space => {
                    if self.game.settings.debug_mode {
                        self.game.update(self.game.dt);
                    }
                }
                Key::D => self.game.settings.debug_mode = !self.game.settings.debug_mode,
                _ => {}
            }
        }
    }
}