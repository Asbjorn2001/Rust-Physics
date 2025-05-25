use graphics::Transformed;

use crate::game::game_controller::*;
use crate::GlyphCache;
use crate::Texture;
use crate::GlGraphics;
use crate::Context;
use crate::Vector2f;

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