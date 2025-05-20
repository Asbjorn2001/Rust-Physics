extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate kira;
extern crate piston_window;

mod game;
mod utils;
mod physics;
mod user_interface;

use graphics::rectangle::Border;
use physics::shape_type::ShapeType;
use piston_window::{Filter, TextureSettings};
use utils::vector2f::Vector2f;
use physics::polygon::Polygon;
use physics::rigid_body::{RigidBody, BASE_DYNAMIC_FRICTION, BASE_ELASTICITY, BASE_STATIC_FRICTION};
use user_interface::components::*;
use glutin_window::GlutinWindow as Window;
use graphics::*;
use kira::{AudioManager, AudioManagerSettings, DefaultBackend};
use opengl_graphics::{GlGraphics, OpenGL, Texture};
use piston::event_loop::{EventSettings, Events};
use piston::input::*;
use piston::window::WindowSettings;
use std::{clone, vec};
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
        .exit_on_esc(true)
        .resizable(false)
        .build()
        .unwrap();
    
    let mut gl = GlGraphics::new(opengl);

    let args: Vec<_> = std::env::args().collect();

    // Load the font
    let ts = TextureSettings::new().filter(Filter::Nearest);
    let mut glyphs: GlyphCache<'static, (), Texture> = GlyphCache::new(FONT, (), ts).unwrap();
    
    // Create bodies
    let verts = vec![
        Vector2f::new(-15.0, 15.0),
        Vector2f::new(-15.0, 0.0),
        Vector2f::new(0.0, -15.0),
        Vector2f::new(15.0, 0.0),
        Vector2f::new(15.0, 15.0) 
    ];

    let player_body = RigidBody::new(
        ShapeType::Polygon(Polygon::new(
            verts,
            Vector2f::new(250.0, 250.0), 
            color::PURPLE
        )),
        4.0, 
        BASE_ELASTICITY, 
        BASE_STATIC_FRICTION, 
        BASE_DYNAMIC_FRICTION, 
        false,
    );

    let floor_shape = ShapeType::Polygon(Polygon::new_rectangle(
        Vector2f::new(640.0, 650.0), 
        800.0, 
        50.0, 
        color::OLIVE
    ));
    let floor = RigidBody::new(floor_shape, 1.0, BASE_ELASTICITY, BASE_STATIC_FRICTION, BASE_DYNAMIC_FRICTION, true);

    let mut ramp_shape1 = ShapeType::Polygon(Polygon::new_rectangle(
        Vector2f::new(450.0, 400.0), 
        400.0, 
        25.0, 
        color::TEAL
    ));
    let mut ramp_shape2 = ramp_shape1.clone();

    ramp_shape1.rotate(0.5);

    ramp_shape2.translate(Vector2f::new(400.0, -200.0));
    ramp_shape2.rotate(-0.5);
    ramp_shape2.set_color(color::MAROON);

    let ramp1 = RigidBody::new(ramp_shape1, 1.0, BASE_ELASTICITY, BASE_STATIC_FRICTION, BASE_DYNAMIC_FRICTION, true); 
    let ramp2 = RigidBody::new(ramp_shape2, 1.0, BASE_ELASTICITY, BASE_STATIC_FRICTION, BASE_DYNAMIC_FRICTION, true);

    let audio_manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()).unwrap();

    let game = Game {
        settings: GameSettings::default(),
        enable_launch: true,
        bodies: vec![floor, ramp1, ramp2],
        target: None,
        projectile: player_body,
        contacts: vec![],
    };

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
