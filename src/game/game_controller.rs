use crate::game::Game;
use crate::game_states::*;
use crate::Vector2f;
use crate::piston::*;

pub struct GameController {
    pub game: Game,
    pub state: Box<dyn GameState>,
    pub cursor_pos: Vector2f<f64>,
}

impl GameController {
    pub fn new(game: Game) -> Self {
        Self { 
            state: Box::new(main_menu::MainMenu::from(&game)),
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
    }
}