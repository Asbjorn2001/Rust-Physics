use crate::game_state::gui_component::*;
use crate::Vector2f;
use crate::GlyphCache;
use crate::game_state::GameState;
use crate::game_state::*;
use crate::Texture;
use piston_window::*;
use super::playing_state::PlayingState;
use super::gui::GUI;


pub struct PauseState {
    pause_menu: GUI,
    settings_menu: GUI,
    open_settings: bool,
}

impl GameState for PauseState {
    fn draw(&self, game: &Game, glyphs: &mut GlyphCache<'static, (), Texture>, c: Context, gl: &mut GlGraphics) {
        game.draw(glyphs, c, gl);

        let dims = c.get_view_size();
        let rect = [0.0, 0.0, dims[0], dims[1]];
        let gray = [1.0, 1.0, 1.0, 0.5];
        graphics::rectangle(gray, rect, c.transform, gl); 

        if self.open_settings {
            self.settings_menu.draw(glyphs, c, gl);
        } else {
            self.pause_menu.draw(glyphs, c, gl);
        }
    }

    fn update(&mut self, control_args: &ControlArgs, e: &Event, game: &mut Game) -> Option<Box<dyn GameState>>{
        let mut next_state = None;
        if self.open_settings {
            for component in self.settings_menu.components.as_mut_slice() {
                match component.update(control_args.cursor_pos(), e, game) {
                    GUIEvent::Custom(event) => {
                        if event.as_str() == "back" {
                            self.open_settings = false;
                        }
                    }
                    _ => {},
                }
            }
        } else {
            for component in self.pause_menu.components.as_mut_slice() {
                match component.update(control_args.cursor_pos(), e, game) {
                    GUIEvent::StateChange(state) => next_state = Some(state),
                    GUIEvent::Custom(event) => {
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
                next_state = Some(Box::new(PlayingState::from(&*game)));
            } 
        }
        
        next_state
    }
}

impl From<&Game> for PauseState {
    fn from(value: &Game) -> Self {  
        let text = Text::new_color(color::BLACK, 20);
        let mut rect = Rectangle::new_round_border(color::BLACK, 15.0, 1.0).color(color::CYAN);
        let mut button_position = Vector2f::new(540.0, 100.0);
        let button_size = Vector2f::new(200.0, 50.0);

        let display = Display::new(rect, DisplayContent::Text(text, "Resume".to_string()));
        let resume_button = GUIButton::new(
            button_position, 
            button_size, 
            display,
            |btn, event, game| { 
                match event {
                    GUIEvent::Click => return GUIEvent::StateChange(Box::new(playing_state::PlayingState::from(&*game))),
                    GUIEvent::Hover => btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 15.0, 2.0).border,
                    GUIEvent::UnHover => btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 15.0, 1.0).border,
                    _ => {}
                }
                event                
            },
        );

        button_position.y += 75.0;
        let display = Display::new(rect, DisplayContent::Text(text, "Settings".to_string()));
        let settings_button = GUIButton::new(
            button_position, 
            button_size, 
            display,
            |btn, event, _| { 
                match event {
                    GUIEvent::Click => return GUIEvent::Custom("open_settings".to_string()),
                    GUIEvent::Hover => btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 15.0, 2.0).border,
                    GUIEvent::UnHover => btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 15.0, 1.0).border,
                    _ => {}
                }
                event
            },
        );

        button_position.y += 75.0;
        let display = Display::new(rect, DisplayContent::Text(text, "Reset".to_string()));
        let reset_button = GUIButton::new(
            button_position, 
            button_size, 
            display,
            |btn, event, game| { 
                match event {
                    GUIEvent::Click => {
                        *game = Game::default();
                        return GUIEvent::StateChange(Box::new(PlayingState::from(&*game)));
                    },
                    GUIEvent::Hover => btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 15.0, 2.0).border,
                    GUIEvent::UnHover => btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 15.0, 1.0).border,
                    _ => {}
                }
                event
            },
        );

        button_position.y += 75.0;
        rect.color = color::RED;
        let display = Display::new(rect, DisplayContent::Text(text, "Exit to menu".to_string()));
        let exit_button = GUIButton::new(
            button_position, 
            button_size, 
            display,
            |btn, event, game| {
                match event {
                    GUIEvent::Click => return GUIEvent::StateChange(Box::new(main_state::MainState::from(&*game))),
                    GUIEvent::Hover => btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 15.0, 2.0).border,
                    GUIEvent::UnHover => btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 15.0, 1.0).border,
                    _ => {}
                }
                event
            }, 
        );

        Self { 
            pause_menu: GUI { components: vec![Box::new(resume_button), Box::new(settings_button), Box::new(reset_button), Box::new(exit_button)] }, 
            settings_menu: GUI::get_settings_menu(value),
            open_settings: false,
        }      
    }
}