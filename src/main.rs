extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate kira;
extern crate piston_window;

mod game;
mod utils;
mod physics;
mod game_state;

use std::path::Path;

use piston_window::{Filter, TextureSettings};
use utils::vector2f::Vector2f;
use glutin_window::GlutinWindow as Window;
use graphics::*;
use kira::{AudioManager, AudioManagerSettings, DefaultBackend, PlaySoundError};
use opengl_graphics::{GlGraphics, OpenGL, Texture};
use piston::event_loop::{EventSettings, Events};
use piston::input::*;
use piston::window::WindowSettings;
use glyph_cache::rusttype::GlyphCache;
use game::*;
use game::game_controller::*;
use game::game_view::*;

static FONT: &str = "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf";

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;
    let start_dims = [1280, 720];

    // Create a Glutin window.
    let mut window: Window = WindowSettings::new("piston-game", start_dims)
        .graphics_api(opengl)
        .resizable(false)
        .build()
        .unwrap();
    
    let mut gl = GlGraphics::new(opengl);

    let args: Vec<_> = std::env::args().collect();

    // Load the font
    let ts = TextureSettings::new().filter(Filter::Nearest);
    let mut glyphs: GlyphCache<'static, (), Texture> = GlyphCache::new(FONT, (), ts).unwrap();
    
    let audio_manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()).unwrap();

    let game = Game::default();
    let mut game_controller = GameController::new(game);
    let mut game_view = GameView::new();

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        
        game_controller.event(&e);

        if let Some(args) = e.render_args() {
            gl.draw(args.viewport(), |c, g| {
                graphics::clear(color::WHITE, g);
                
                game_view.draw(&game_controller, &mut glyphs, c, g);
            });
        }
    }
}
