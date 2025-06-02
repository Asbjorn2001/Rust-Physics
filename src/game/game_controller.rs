use std::collections::HashMap;

use crate::game::Game;
use crate::game_state::*;
use crate::Vector2f;
use piston_window::*;

use super::MAX_SCALE;
use super::MIN_SCALE;

pub struct ControlArgs {
    cursor_pos: Vector2f<f64>,
    pressed_buttons: HashMap<Button, bool>,
}

impl ControlArgs {
    pub fn cursor_pos(&self) -> Vector2f<f64> {
        return self.cursor_pos;
    }

    pub fn button_pressed(&self, button: &Button) -> bool {
        if let Some(button) = self.pressed_buttons.get(button) {
            return *button;
        }
        return false;
    }
}

pub struct GameController {
    pub game: Game,
    pub state: Box<dyn GameState>,
    pub control_args: ControlArgs,
}

#[allow(dead_code)]
impl GameController {
    pub fn new(game: Game) -> Self {
        Self { 
            state: Box::new(main_state::MainState::from(&game)),
            game: game, 
            control_args: ControlArgs { 
                cursor_pos: Vector2f::zero(), 
                pressed_buttons: HashMap::new() 
            }
        }
    }

    pub fn event(&mut self, e: &Event) {
        if let Some(pos) = e.mouse_cursor_args() {
            self.control_args.cursor_pos = pos.into();
        }

        if let Some(button) = e.press_args() {
            self.control_args.pressed_buttons.entry(button).and_modify(|value| *value = true).or_insert(true);
        }

        if let Some(button) = e.release_args() {
            self.control_args.pressed_buttons.entry(button).and_modify(|value| *value = false).or_insert(false);
        }

        if !self.game.settings.debug_mode {
            if let Some(new_state) = self.state.update(&self.control_args, e, &mut self.game) {
                self.state = new_state;
            }
        }

        if self.control_args.button_pressed(&Button::Keyboard(Key::W)) { self.game.settings.camera.scale = MAX_SCALE.min(self.game.settings.camera.scale * 1.01); }
        if self.control_args.button_pressed(&Button::Keyboard(Key::S)) { self.game.settings.camera.scale = MIN_SCALE.max(self.game.settings.camera.scale * 1.0 / 1.01); }
        if self.control_args.button_pressed(&Button::Keyboard(Key::Up)) { self.game.settings.camera.position.y -= 5.0; }
        if self.control_args.button_pressed(&Button::Keyboard(Key::Down)) { self.game.settings.camera.position.y += 5.0; }
        if self.control_args.button_pressed(&Button::Keyboard(Key::Left)) { self.game.settings.camera.position.x -= 5.0; }
        if self.control_args.button_pressed(&Button::Keyboard(Key::Right)) { self.game.settings.camera.position.x += 5.0; }

        if let Some(Button::Keyboard(key)) = e.press_args() {
            match key {
                Key::Space => {
                    if self.game.settings.debug_mode {
                        self.game.update(self.game.physics.dt);
                    }
                }
                Key::X => self.game.settings.debug_mode = !self.game.settings.debug_mode,
                _ => {}
            }
        }
    }
}