use crate::game_states::components::*;
use crate::Vector2f;
use crate::color;
use crate::Text;
use crate::GlyphCache;
use crate::game_states::GameState;
use crate::*;
use crate::game::Game;
use crate::Texture;
use piston_window::*;


pub struct SettingsMenu {
    components: Vec<Box<dyn UIComponent>>,
}

impl GameState for SettingsMenu {
    fn draw(&self, game: &Game, glyphs: &mut GlyphCache<'static, (), Texture>, c: Context, gl: &mut GlGraphics) {
        for component in self.components.as_slice() {
            component.draw(glyphs, c, gl);
        }
    }

    fn update(&mut self, cursor_pos: Vector2f<f64>, e: &Event, game: &mut Game) -> Option<Box<dyn GameState>>{
        let mut next_state = None;
        for component in self.components.as_mut_slice() {
            match component.update(cursor_pos, e, game) {
                UIEvent::StateChange(state) => next_state = Some(state),
                _ => {}
            }
        }

        next_state
    }
}

impl From<&Game> for SettingsMenu {
    fn from(value: &Game) -> Self {    
        let text = Text::new_color(color::BLACK, 20);
        let mut rect = Rectangle::new_round_border(color::BLACK, 15.0, 1.0).color(color::GREEN);
        let mut button_position = Vector2f::new(540.0, 100.0);
        let button_size = Vector2f::new(200.0, 50.0);

        rect.color = if value.settings.enable_launch { color::GREEN } else { color::RED };
        let display = Display::new(rect, DisplayContent::Text((text, "Enable launch".to_string())));
        let button1 = UIButton::new(
            button_position, 
            button_size, 
            display,
            |btn, event, game| { 
                match event {
                    UIEvent::Click => {
                        game.settings.enable_launch = !game.settings.enable_launch; 
                        btn.display.rect.color = if game.settings.enable_launch { color::GREEN } else { color::RED };
                    }
                    UIEvent::Hover => btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 15.0, 2.0).border,
                    UIEvent::UnHover => btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 15.0, 1.0).border,
                    _ => {}
                }
                event                
            },
        );

        button_position.y += 75.0;
        rect.color = if value.settings.view.show_velocites { color::GREEN } else { color::RED };
        let display = Display::new(rect, DisplayContent::Text((text, "Show velocity".to_string())));
        let button2 = UIButton::new(
            button_position, 
            button_size, 
            display,
            |btn, event, game| { 
                match event {
                    UIEvent::Click => {
                        game.settings.view.show_velocites = !game.settings.view.show_velocites; 
                        btn.display.rect.color = if game.settings.view.show_velocites { color::GREEN } else { color::RED };
                    }
                    UIEvent::Hover => btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 15.0, 2.0).border,
                    UIEvent::UnHover => btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 15.0, 1.0).border,
                    _ => {}
                }
                event                
            },
        );

        button_position.y += 75.0;
        rect.color = if value.settings.view.show_contact_points { color::GREEN } else { color::RED };
        let display = Display::new(rect, DisplayContent::Text((text, "Show contacts".to_string())));
        let button3 = UIButton::new(
            button_position, 
            button_size, 
            display,
            |btn, event, game| { 
                match event {
                    UIEvent::Click => {
                        game.settings.view.show_contact_points = !game.settings.view.show_contact_points;
                        btn.display.rect.color = if game.settings.view.show_contact_points { color::GREEN } else { color::RED };
                    }
                    UIEvent::Hover => btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 15.0, 2.0).border,
                    UIEvent::UnHover => btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 15.0, 1.0).border,
                    _ => {}
                }
                event
            },
        );

        Self { 
            components: vec![
                Box::new(button1),
                Box::new(button2),
                Box::new(button3),
            ] 
        }
    }
}