use super::gui_component::*;
use crate::GlyphCache;
use crate::Texture;
use crate::GlGraphics;
use crate::color;
use crate::Vector2f;
use piston_window::*;
use crate::game::Game;

pub struct GUI {
    pub components: Vec<Box<dyn GUIComponent>>,
}

impl GUI {
    pub fn draw(&self, glyphs: &mut GlyphCache<'static, (), Texture>, c: Context, gl: &mut GlGraphics) {
        for component in self.components.as_slice() {
            component.draw(glyphs, c, gl);
        }
    }

    pub fn get_settings_menu(model: &Game) -> Self {
        let text = Text::new_color(color::BLACK, 20);
        let mut rect = Rectangle::new_round_border(color::BLACK, 15.0, 1.0).color(color::CYAN);
        let mut button_position = Vector2f::new(540.0, 100.0);
        let button_size = Vector2f::new(200.0, 50.0);

        let display = Display::new(rect, DisplayContent::Text((text, "Back".to_string())));
        let back_button = GUIButton::new(
            button_position, 
            button_size, 
            display,
            |btn, event, game| { 
                match event {
                    GUIEvent::Click => return GUIEvent::Custom("back".to_string()),
                    GUIEvent::Hover => btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 15.0, 2.0).border,
                    GUIEvent::UnHover => btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 15.0, 1.0).border,
                    _ => {}
                }
                event                
            },
        );

        button_position.y += 75.0;
        rect.color = if model.settings.enable_launch { color::CYAN } else { color::RED };
        let display = Display::new(rect, DisplayContent::Text((text, "Enable launch".to_string())));
        let enable_launch_button = GUIButton::new(
            button_position, 
            button_size, 
            display,
            |btn, event, game| { 
                match event {
                    GUIEvent::Click => {
                        game.settings.enable_launch = !game.settings.enable_launch; 
                        btn.display.rect.color = if game.settings.enable_launch { color::CYAN } else { color::RED };
                    }
                    GUIEvent::Hover => btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 15.0, 2.0).border,
                    GUIEvent::UnHover => btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 15.0, 1.0).border,
                    _ => {}
                }
                event                
            },
        );

        button_position.y += 75.0;
        rect.color = if model.settings.view.show_velocites { color::CYAN } else { color::RED };
        let display = Display::new(rect, DisplayContent::Text((text, "Show velocity".to_string())));
        let show_velocity_button = GUIButton::new(
            button_position, 
            button_size, 
            display,
            |btn, event, game| { 
                match event {
                    GUIEvent::Click => {
                        game.settings.view.show_velocites = !game.settings.view.show_velocites; 
                        btn.display.rect.color = if game.settings.view.show_velocites { color::CYAN } else { color::RED };
                    }
                    GUIEvent::Hover => btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 15.0, 2.0).border,
                    GUIEvent::UnHover => btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 15.0, 1.0).border,
                    _ => {}
                }
                event                
            },
        );

        button_position.y += 75.0;
        rect.color = if model.settings.view.show_contact_points { color::CYAN } else { color::RED };
        let display = Display::new(rect, DisplayContent::Text((text, "Show contacts".to_string())));
        let show_contacts_button = GUIButton::new(
            button_position, 
            button_size, 
            display,
            |btn, event, game| { 
                match event {
                    GUIEvent::Click => {
                        game.settings.view.show_contact_points = !game.settings.view.show_contact_points;
                        btn.display.rect.color = if game.settings.view.show_contact_points { color::CYAN } else { color::RED };
                    }
                    GUIEvent::Hover => btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 15.0, 2.0).border,
                    GUIEvent::UnHover => btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 15.0, 1.0).border,
                    _ => {}
                }
                event
            },
        );

        button_position.y += 75.0;
        rect.color = if model.settings.view.show_tiles { color::CYAN } else { color::RED };
        let display = Display::new(rect, DisplayContent::Text((text, "Show tiles".to_string())));
        let show_tiles_button = GUIButton::new(
            button_position, 
            button_size, 
            display,
            |btn, event, game| { 
                match event {
                    GUIEvent::Click => {
                        game.settings.view.show_tiles = !game.settings.view.show_tiles;
                        btn.display.rect.color = if game.settings.view.show_tiles { color::CYAN } else { color::RED };
                    }
                    GUIEvent::Hover => btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 15.0, 2.0).border,
                    GUIEvent::UnHover => btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 15.0, 1.0).border,
                    _ => {}
                }
                event
            },
        );

        Self { 
            components: vec![
                Box::new(back_button),
                Box::new(enable_launch_button),
                Box::new(show_velocity_button),
                Box::new(show_contacts_button),
                Box::new(show_tiles_button),
            ] 
        }
    }
}