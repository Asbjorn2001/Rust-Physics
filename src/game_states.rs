pub mod components;
pub mod playing;
pub mod settings;
pub mod main_menu;

use crate::Vector2f;
use crate::Game;
use crate::piston::*;
use crate::graphics::*;
use crate::opengl_graphics::*;
use crate::GlyphCache;

pub trait GameState {
    fn draw(&self, game: &Game, glyphs: &mut GlyphCache<'static, (), Texture>, c: Context, gl: &mut GlGraphics);

    fn update(&mut self, cursor_pos: Vector2f<f64>, e: &Event, game: &mut Game) -> Option<Box<dyn GameState>>;
}