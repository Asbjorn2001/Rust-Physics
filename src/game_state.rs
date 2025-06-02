pub mod gui_component;
pub mod playing_state;
pub mod main_state;
pub mod pause_state;
pub mod gui;

use crate::game::game_controller::ControlArgs;
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
    fn update(&mut self, control_args: &ControlArgs, e: &Event, game: &mut Game) -> Option<Box<dyn GameState>>;
}