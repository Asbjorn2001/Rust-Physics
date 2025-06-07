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

use piston_window::{Filter, TextureSettings};
use utils::vector2f::Vector2f;
use glutin_window::GlutinWindow as Window;
use graphics::*;
use kira::{AudioManager, AudioManagerSettings, DefaultBackend};
use opengl_graphics::{GlGraphics, OpenGL, Texture};
use piston::event_loop::{EventSettings, Events};
use piston::input::*;
use piston::window::WindowSettings;
use glyph_cache::rusttype::GlyphCache;
use game::*;
use game::game_controller::*;
use game::game_view::*;

static FONT: &str = "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf";

pub const WINDOW_WIDTH: f64 = 1280.0;
pub const WINDOW_HEIGHT: f64 = 720.0;

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create a Glutin window.
    let mut window: Window = WindowSettings::new("physics-playground", [WINDOW_WIDTH, WINDOW_HEIGHT])
        .graphics_api(opengl)
        .resizable(false)
        .build()
        .unwrap();
    
    let mut gl = GlGraphics::new(opengl);

    let _: Vec<_> = std::env::args().collect();

    // Load the font
    let ts = TextureSettings::new().filter(Filter::Nearest);
    let mut glyphs: GlyphCache<'static, (), Texture> = GlyphCache::new(FONT, (), ts).unwrap();
    
    let _ = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()).unwrap();

    let game = Game::default();
    let mut game_controller = GameController::new(game);
    let mut game_view = GameView::new();

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        
        game_controller.event(&e);
        
        if let Some(args) = e.render_args() {
            gl.draw(args.viewport(), |c, gl| {
                graphics::clear(color::WHITE, gl);
                
                game_controller.game.update_camera(c);

                game_view.draw(&game_controller, &mut glyphs, c, gl);
            });
        }
    }
}
