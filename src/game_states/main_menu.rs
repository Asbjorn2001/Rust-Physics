use crate::game_states::components::*;
use crate::Vector2f;
use crate::Game;
use crate::piston::*;
use crate::graphics::*;
use crate::opengl_graphics::*;
use crate::GlyphCache;
use crate::game_states::GameState;


pub struct MainMenu {
    components: Vec<Box<dyn UIComponent>>
}

impl GameState for MainMenu {
    fn draw(&self, game: &Game, glyphs: &mut GlyphCache<'static, (), Texture>, c: Context, gl: &mut GlGraphics) {
        for component in self.components.as_slice() {
            component.draw(glyphs, c, gl);
        }
    }

    fn update(&mut self, cursor_pos: Vector2f<f64>, e: &Event, game: &mut Game) -> Option<Box<dyn GameState>>{
        for component in self.components.as_mut_slice() {
            component.update(cursor_pos, e, game);
        }

        None
    }
}

impl From<&Game> for MainMenu {
    fn from(value: &Game) -> Self {        
        Self { 
            components: vec![

            ] 
        }
    }
}