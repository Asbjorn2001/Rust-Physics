use crate::game_states::components::*;
use crate::Vector2f;
use crate::GlyphCache;
use crate::game_states::GameState;
use crate::game_states::*;
use crate::Texture;
use piston_window::*;

use super::playing::Playing;
use super::settings::SettingsMenu;


pub struct PauseMenu {
    components: Vec<Box<dyn UIComponent>>,
    settings_menu: SettingsMenu,
    open_settings: bool,
}

impl GameState for PauseMenu {
    fn draw(&self, game: &Game, glyphs: &mut GlyphCache<'static, (), Texture>, c: Context, gl: &mut GlGraphics) {
        game.draw(glyphs, c, gl);

        let dims = c.get_view_size();
        let rect = [0.0, 0.0, dims[0], dims[1]];
        let gray = [1.0, 1.0, 1.0, 0.5];
        graphics::rectangle(gray, rect, c.transform, gl); 

        if self.open_settings {
            self.settings_menu.draw(game, glyphs, c, gl);
        } else {
            for component in self.components.as_slice() {
                component.draw(glyphs, c, gl);
            }
        }
    }

    fn update(&mut self, cursor_pos: Vector2f<f64>, e: &Event, game: &mut Game) -> Option<Box<dyn GameState>>{
        let mut next_state = None;
        if self.open_settings {
            next_state = self.settings_menu.update(cursor_pos, e, game);
        } else {
            for component in self.components.as_mut_slice() {
                match component.update(cursor_pos, e, game) {
                    UIEvent::StateChange(state) => next_state = Some(state),
                    UIEvent::Custom(event) => {
                        if event.as_str() == "open_settings" {
                            self.open_settings = true;
                        }
                    },
                    _ => {},
                }
            }
        }

        if let Some(Button::Keyboard(Key::Escape)) = e.press_args() {
            if self.open_settings {
                self.open_settings = false;
            } else {
                next_state = Some(Box::new(Playing::from(&*game)));
            } 
        }
        
        next_state
    }
}

impl From<&Game> for PauseMenu {
    fn from(value: &Game) -> Self {  
        let text = Text::new_color(color::BLACK, 20);
        let mut rect = Rectangle::new_round_border(color::BLUE, 15.0, 1.0).color(color::GREEN);
        let mut button_position = Vector2f::new(540.0, 100.0);
        let button_size = Vector2f::new(200.0, 50.0);

        let display = Display::new(rect, DisplayContent::Text((text, "Resume".to_string())));
        let button1 = UIButton::new(
            button_position, 
            button_size, 
            display,
            |btn, event, game| { 
                match event {
                    UIEvent::Click => return UIEvent::StateChange(Box::new(playing::Playing::from(&*game))),
                    UIEvent::Hover => btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 15.0, 2.0).border,
                    UIEvent::UnHover => btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 15.0, 1.0).border,
                    _ => {}
                }
                event                
            },
        );

        button_position.y += 75.0;
        let display = Display::new(rect, DisplayContent::Text((text, "Settings".to_string())));
        let button2 = UIButton::new(
            button_position, 
            button_size, 
            display,
            |btn, event, game| { 
                match event {
                    UIEvent::Click => return UIEvent::Custom("open_settings".to_string()),
                    UIEvent::Hover => btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 15.0, 2.0).border,
                    UIEvent::UnHover => btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 15.0, 1.0).border,
                    _ => {}
                }
                event
            },
        );

        button_position.y += 75.0;
        rect.color = color::RED;
        let display = Display::new(rect, DisplayContent::Text((text, "Exit".to_string())));
        let button3 = UIButton::new(
            button_position, 
            button_size, 
            display,
            |btn, event, game| {
                match event {
                    UIEvent::Click => return UIEvent::StateChange(Box::new(main_menu::MainMenu::from(&*game))),
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
            ],
            settings_menu: SettingsMenu::from(value),
            open_settings: false,
        }      
    }
}