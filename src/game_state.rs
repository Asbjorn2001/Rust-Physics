pub mod gui_component;
pub mod playing_state;
pub mod main_state;
pub mod pause_state;
pub mod gui;

use crate::Vector2f;
use crate::game::*;
use crate::glyph_cache::rusttype::GlyphCache;
use crate::Texture;
use crate::GlGraphics;
use crate::Context;
use piston_window::Event;

pub trait GameState {
    #[allow(dead_code)]
    fn draw(&self, game: &Game, glyphs: &mut GlyphCache<'static, (), Texture>, c: Context, gl: &mut GlGraphics);

    #[allow(dead_code)]
    fn update(&mut self, cursor_pos: Vector2f<f64>, e: &Event, game: &mut Game) -> Option<Box<dyn GameState>>;
}

