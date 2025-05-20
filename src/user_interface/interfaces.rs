use graphics::ellipse::Border;

use crate::game::game_controller::GameController;
use crate::user_interface::components::*;
use crate::Vector2f;
use crate::Game;
use crate::piston::*;
use crate::color;
use crate::graphics::*;
use crate::Text;
use crate::opengl_graphics::*;
use crate::GlyphCache;

trait GameState {
    fn draw(&self, controller: &GameController, glyphs: &mut GlyphCache<'static, (), Texture>, c: Context, gl: &mut GlGraphics);

    fn update(&mut self, cursor_pos: Vector2f<f64>, e: &Event, controller: &mut GameController) -> bool;
}

struct Playing {
    components: Vec<Box<dyn UIComponent>>
}

impl GameState for Playing {
    fn draw(&self, controller: &GameController, glyphs: &mut GlyphCache<'static, (), Texture>, c: Context, gl: &mut GlGraphics) {
        
    }   

    fn update(&mut self, cursor_pos: Vector2f<f64>, e: &Event, controller: &mut GameController) -> bool {
        true
    }
}

struct SettingsMenu {
    components: Vec<Box<dyn UIComponent>>
}

struct Inventory {
    components: Vec<Box<dyn UIComponent>>
}

struct MainMenu {
    components: Vec<Box<dyn UIComponent>>
}

struct PauseMenu {
    components: Vec<Box<dyn UIComponent>>
}

pub struct Interfaces {
    pub game: UIMenu,
    pub settings: UIMenu,
}

impl Interfaces {
    pub fn initialize() -> Self {
        Self { 
            game: UIMenu::game_interface(), 
            settings: UIMenu::settings_interface(), 
        }
    }
}

pub struct UIMenu {
    components: Vec<Box<dyn UIComponent>>
}

impl UIComponent for UIMenu {
    fn draw(&self, glyphs: &mut GlyphCache<'static, (), Texture>, c: Context, gl: &mut GlGraphics) {
        let dims = c.get_view_size();
        let rect = [0.0, 0.0, dims[0], dims[1]];
        let grayed = [1.0, 1.0, 1.0, 0.5];
        //graphics::rectangle(grayed, rect, c.transform, gl);       
        for component in self.components.as_slice() {
            component.draw(glyphs, c, gl);
        }
    }

    fn update(&mut self, cursor_pos: Vector2f<f64>, e: &Event, game: &mut Game) -> bool {
        let mut interaction = false;
        for component in self.components.as_mut_slice() {
            if component.update(cursor_pos, e, game) {
                interaction = true;
            }
        }

        interaction
    }
}

impl UIMenu {
    pub fn game_interface() -> Self {
        let gravity_slider = UISlider2D::new(Vector2f::new(25.0, 25.0), 200.0, |value, game| {
            game.settings.physics.gravity = value * 500.0;
        });

        let rect = Rectangle::new_round_border(color::BLACK, 5.0, 2.0);
        let text = Text::new(20);
        let text_box = TextBox::new(rect, text, "G".to_string());
        let gravity_display = UIDisplay::new(
            Vector2f::new(25.0, 250.0), 
            Vector2f::new(200.0, 50.0), 
            text_box,
            |display, game| {
                display.text_box.string = format!("G: {:.2} m/s²", game.settings.physics.gravity.len() / 100.0);
            } 
        );

        Self { 
            components: vec![
                Box::new(gravity_slider),
                Box::new(gravity_display,)
            ] 
        }    
    }

    pub fn settings_interface() -> Self {
        let text = Text::new_color(color::BLACK, 20);
        let rect = Rectangle::new_round_border(color::BLACK, 15.0, 1.0).color(color::GREEN);
        let mut button_box = TextBox::new(rect, text, String::new());

        let mut button_position = Vector2f::new(200.0, 100.0);
        let button_size = Vector2f::new(200.0, 50.0);
        
        button_box.string = "Enable launch".to_string();
        let button1 = UIButton::new(
            button_position, 
            button_size, 
            button_box.clone(), 
            |btn, game| { game.enable_launch = !game.enable_launch; },
            |btn, game| { 
                btn.display.rect.color = if game.enable_launch { color::GREEN } else { color::RED };
                btn.display.rect.border = if btn.is_hovered { 
                    Rectangle::new_round_border(color::BLACK, 15.0, 2.0).border 
                } else {
                    Rectangle::new_round_border(color::BLACK, 15.0, 1.0).border
                }
            },
        );

        button_position.y += 75.0;
        button_box.string = "Show velocity".to_string();
        let button2 = UIButton::new(
            button_position, 
            button_size, 
            button_box.clone(), 
            |btn, game| { game.settings.view.show_velocites = !game.settings.view.show_velocites; },
            |btn, game| { 
                btn.display.rect.color = if game.settings.view.show_velocites { color::GREEN } else { color::RED };
                btn.display.rect.border = if btn.is_hovered { 
                    Rectangle::new_round_border(color::BLACK, 15.0, 2.0).border 
                } else {
                    Rectangle::new_round_border(color::BLACK, 15.0, 1.0).border
                }
            }, 
        );

        button_position.y += 75.0;
        button_box.string = "Show contacts".to_string();
        let button3 = UIButton::new(
            button_position, 
            button_size, 
            button_box.clone(), 
            |btn, game| { game.settings.view.show_contact_points = !game.settings.view.show_contact_points; },
            |btn, game| { 
                btn.display.rect.color = if game.settings.view.show_contact_points { color::GREEN } else { color::RED };
                btn.display.rect.border = if btn.is_hovered { 
                    Rectangle::new_round_border(color::BLACK, 15.0, 2.0).border 
                } else {
                    Rectangle::new_round_border(color::BLACK, 15.0, 1.0).border
                }
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