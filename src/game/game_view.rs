use crate::game::game_controller::*;
use crate::physics::shape::Renderable;
use crate::graphics::*;
use crate::user_interface::components::UIComponent;
use crate::user_interface::interfaces::GameState;
use crate::GlyphCache;
use crate::Texture;
use crate::GlGraphics;

pub struct GameViewSettings {

}

pub struct GameView {
    settings: GameViewSettings,   
}

impl GameView {
    pub fn new() -> Self {
        Self { settings: GameViewSettings {} }
    }

    pub fn draw(&mut self, controller: &GameController, glyphs: &mut GlyphCache<'static, (), Texture>, c: Context, gl: &mut GlGraphics) {
        controller.state.draw(&controller.game, glyphs, c, gl);
    }
}