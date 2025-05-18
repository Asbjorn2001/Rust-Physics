use crate::game::game_controller::*;
use crate::physics::shape::Renderable;
use crate::graphics::*;
use crate::user_interface::components::UIComponent;
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

    pub fn draw(&mut self, controller: &GameController, glyphs: &mut GlyphCache<'static, (), Texture>, c: Context, gl: &mut GlGraphics ) {
        graphics::clear(color::WHITE, gl);

        for obj in controller.game.bodies.as_slice() {
            obj.shape.draw(c, gl);
            if controller.game.settings.view.show_velocites {
                let o = obj.shape.get_center();
                let vel = obj.linear_velocity;
                let line = [o.x, o.y, o.x + vel.x, o.y + vel.y];
                graphics::line(color::CYAN, 1.0, line, c.transform, gl);
            }
        }

        if controller.game.settings.view.show_contact_points {
            for cp in controller.game.contacts.as_slice() {
                let square = graphics::rectangle::centered_square(cp.x, cp.y, 5.0);
                graphics::ellipse(color::YELLOW, square, c.transform, gl);
            }
        }
        
        if let Some(target) = controller.game.target {
            if !controller.game.enable_launch {
                return;
            }
            let line = [controller.cursor_pos.x, controller.cursor_pos.y, target.x, target.y];
            graphics::line(controller.game.projectile.shape.get_color(), 1.0, line, c.transform, gl);
            controller.game.projectile.shape.draw(c, gl);
        }

        match controller.mode {
            UIMode::Game => controller.interfaces.game.draw(glyphs, c, gl),
            UIMode::Settings => controller.interfaces.settings.draw(glyphs, c, gl),
        }
    }
}