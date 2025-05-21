use crate::physics::rigid_body::RigidBody;
use crate::game_states::components::*;
use crate::Vector2f;
use crate::Game;
use crate::piston::*;
use crate::color;
use crate::graphics::*;
use crate::Text;
use crate::opengl_graphics::*;
use crate::GlyphCache;
use crate::physics::shape::Renderable;
use crate::game_states::GameState;
use crate::game_states::settings::SettingsMenu;
use crate::game_states::inventory::Inventory;


pub struct Playing {
    pub components: Vec<Box<dyn UIComponent>>,
}

impl GameState for Playing {
    fn draw(&self, game: &Game, glyphs: &mut GlyphCache<'static, (), Texture>, c: Context, gl: &mut GlGraphics) {
        game.draw(glyphs, c, gl);
        if let Some(target) = game.target {
            if game.settings.enable_launch {
                let projectile_pos = game.projectile.get_center();
                let line = [projectile_pos.x, projectile_pos.y, target.x, target.y];
                graphics::line(game.projectile.get_color(), 1.0, line, c.transform, gl);
                game.projectile.scale(game.projectile_scale).draw(c.transform, gl);
            }
        }
        for component in self.components.as_slice() {
            component.draw(glyphs, c, gl);
        }
    }   

    fn update(&mut self, cursor_pos: Vector2f<f64>, e: &Event, game: &mut Game) -> Option<Box<dyn GameState>> {
        let mut interaction = false;
        let mut next_state = None;
        for component in self.components.as_mut_slice() {
            match component.update(cursor_pos, e, game) {
                UIEvent::None => {},
                UIEvent::StateChange(state) => { 
                    next_state = Some(state);
                    interaction = true;
                }
                UIEvent::Quit => std::process::exit(0),
                _ => interaction = true,
            }
        }
        
        // Update game logic
        if let Some(args) = e.update_args() {
            game.update(&args);
        }

        // Set target on press
        if let Some(Button::Mouse(MouseButton::Left)) = e.press_args() {
            if game.settings.enable_launch && !interaction {
                game.target = Some(cursor_pos);
                let mut color = game.projectile.get_color();
                color[3] = 0.5; // Transparency
                game.projectile.set_color(color);
            } 
        }

        // Launch on release
        if let Some(target) = game.target {
            game.projectile.set_center(cursor_pos);
            if let Some(Button::Mouse(MouseButton::Left)) = e.release_args() {
                let velocity = (target - cursor_pos) * 2.0;

                let mut color = game.projectile.get_color();
                color[3] = 1.0; // Back to solid
                game.projectile.set_color(color);
                let mut body = RigidBody::from(game.projectile.scale(game.projectile_scale));
                body.linear_velocity = velocity;
                game.bodies.push(body);

                game.target = None;
            }
        }

        if let Some(Button::Keyboard(key)) = e.press_args() {
            match key {
                Key::Escape => next_state = Some(Box::new(SettingsMenu::from(&*game))),
                Key::E => next_state = Some(Box::new(Inventory::from(&*game))),
                _ => {}
            }
        }

        if next_state.is_some() {
            game.target = None;
        }

        next_state
    }
}

impl From<&Game> for Playing {
    fn from(value: &Game) -> Self {
        let mut gravity_slider = UISlider2D::new(Vector2f::new(25.0, 25.0), 200.0, |value, event, game| {
            match event {
                UIEvent::Change => game.settings.physics.gravity = value * 500.0,
                _ => {}
            }
            event
        });
        gravity_slider.value = value.settings.physics.gravity / 500.0;

        let rect = Rectangle::new_round_border(color::BLACK, 5.0, 2.0);
        let text = Text::new(20);
        let text_box = Display::new(rect, DisplayContent::Text((text, "G".to_string())));
        let gravity_display = UIButton::new(
            Vector2f::new(25.0, 250.0), 
            Vector2f::new(200.0, 50.0), 
            text_box,
            |btn, _, game| {
                if let DisplayContent::Text(text) = &mut btn.display.content {
                    text.1 = format!("G: {:.2} m/s²", game.settings.physics.gravity.len() / 100.0);
                }
                UIEvent::None
            } 
        );

        Self { 
            components: vec![
                Box::new(gravity_slider),
                Box::new(gravity_display)
            ],
        }   
    }
}