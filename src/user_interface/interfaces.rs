use crate::user_interface::components::*;
use crate::Vector2f;
use crate::Game;
use crate::piston::*;
use crate::color;
use crate::graphics::*;
use crate::Text;
use crate::opengl_graphics::*;
use crate::GlyphCache;
use crate::physics::shape::Renderable;

pub trait GameState {
    fn draw(&self, game: &Game, glyphs: &mut GlyphCache<'static, (), Texture>, c: Context, gl: &mut GlGraphics);

    fn update(&mut self, cursor_pos: Vector2f<f64>, e: &Event, game: &mut Game) -> Option<Box<dyn GameState>>;
}

pub struct Playing {
    pub components: Vec<Box<dyn UIComponent>>,
}

impl GameState for Playing {
    fn draw(&self, game: &Game, glyphs: &mut GlyphCache<'static, (), Texture>, c: Context, gl: &mut GlGraphics) {
        game.draw(glyphs, c, gl);
        if let Some(target) = game.target {
            if game.settings.enable_launch {
                let projectile_pos = game.projectile.shape.get_center();
                let line = [projectile_pos.x, projectile_pos.y, target.x, target.y];
                graphics::line(game.projectile.shape.get_color(), 1.0, line, c.transform, gl);
                game.projectile.shape.draw(c, gl);
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
                UIEvent::Some => interaction = true,
                UIEvent::StateChange(state) => { 
                    next_state = Some(state);
                    interaction = true;
                }
                UIEvent::Quit => std::process::exit(0),
                _ => {}
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
                let mut color = game.projectile.shape.get_color();
                color[3] = 0.5; // Transparency
                game.projectile.shape.set_color(color);
            } 
        }

        // Launch on release
        if let Some(target) = game.target {
            game.projectile.shape.set_center(cursor_pos);
            if let Some(Button::Mouse(MouseButton::Left)) = e.release_args() {
                let velocity = (target - cursor_pos) * 2.0;

                let mut color = game.projectile.shape.get_color();
                color[3] = 1.0; // Back to solid
                game.projectile.shape.set_color(color);
                game.projectile.linear_velocity = velocity;
                game.bodies.push(game.projectile.clone());

                game.target = None;
            }
        }

        if let Some(Button::Keyboard(Key::S)) = e.press_args() {
            next_state = Some(Box::new(SettingsMenu::from(&*game)));
        }

        next_state
    }
}

impl From<&Game> for Playing {
    fn from(value: &Game) -> Self {
        let mut gravity_slider = UISlider2D::new(Vector2f::new(25.0, 25.0), 200.0, |value, game| {
            game.settings.physics.gravity = value * 500.0;
        });
        gravity_slider.value = value.settings.physics.gravity / 500.0;

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
                Box::new(gravity_display)
            ],
        }   
    }
}

pub struct SettingsMenu {
    components: Vec<Box<dyn UIComponent>>
}

impl GameState for SettingsMenu {
    fn draw(&self, game: &Game, glyphs: &mut GlyphCache<'static, (), Texture>, c: Context, gl: &mut GlGraphics) {
        game.draw(glyphs, c, gl);
        let dims = c.get_view_size();
        let rect = [0.0, 0.0, dims[0], dims[1]];
        let grayed = [1.0, 1.0, 1.0, 0.5];
        graphics::rectangle(grayed, rect, c.transform, gl); 

        for component in self.components.as_slice() {
            component.draw(glyphs, c, gl);
        }
    }

    fn update(&mut self, cursor_pos: Vector2f<f64>, e: &Event, game: &mut Game) -> Option<Box<dyn GameState>>{
        for component in self.components.as_mut_slice() {
            component.update(cursor_pos, e, game);
        }

        if let Some(Button::Keyboard(Key::S)) = e.press_args() {
            return Some(Box::new(Playing::from(&*game)));
        }

        None  
    }
}

impl From<&Game> for SettingsMenu {
    fn from(value: &Game) -> Self {    
        let text = Text::new_color(color::BLACK, 20);
        let rect = Rectangle::new_round_border(color::BLACK, 15.0, 1.0).color(color::GREEN);
        let mut button_box = TextBox::new(rect, text, String::new());

        let mut button_position = Vector2f::new(200.0, 100.0);
        let button_size = Vector2f::new(200.0, 50.0);
        
        button_box.string = "Enable launch".to_string();
        let mut button1 = UIButton::new(
            button_position, 
            button_size, 
            button_box.clone(), 
            |btn, game| { 
                game.settings.enable_launch = !game.settings.enable_launch; 
                return UIEvent::Some;
            },
            |btn, game| { 
                btn.display.rect.color = if game.settings.enable_launch { color::GREEN } else { color::RED };
                btn.display.rect.border = if btn.is_hovered { 
                    Rectangle::new_round_border(color::BLACK, 15.0, 2.0).border 
                } else {
                    Rectangle::new_round_border(color::BLACK, 15.0, 1.0).border
                }
            },
        );
        button1.display.rect.color = if value.settings.enable_launch { color::GREEN } else { color::RED };

        button_position.y += 75.0;
        button_box.string = "Show velocity".to_string();
        let mut button2 = UIButton::new(
            button_position, 
            button_size, 
            button_box.clone(), 
            |btn, game| { 
                game.settings.view.show_velocites = !game.settings.view.show_velocites; 
                return UIEvent::Some;
            },
            |btn, game| { 
                btn.display.rect.color = if game.settings.view.show_velocites { color::GREEN } else { color::RED };
                btn.display.rect.border = if btn.is_hovered { 
                    Rectangle::new_round_border(color::BLACK, 15.0, 2.0).border 
                } else {
                    Rectangle::new_round_border(color::BLACK, 15.0, 1.0).border
                }
            }, 
        );
        button2.display.rect.color = if value.settings.view.show_velocites { color::GREEN } else { color::RED };

        button_position.y += 75.0;
        button_box.string = "Show contacts".to_string();
        let mut button3 = UIButton::new(
            button_position, 
            button_size, 
            button_box.clone(), 
            |btn, game| { 
                game.settings.view.show_contact_points = !game.settings.view.show_contact_points; 
                return UIEvent::Some;
            },
            |btn, game| { 
                btn.display.rect.color = if game.settings.view.show_contact_points { color::GREEN } else { color::RED };
                btn.display.rect.border = if btn.is_hovered { 
                    Rectangle::new_round_border(color::BLACK, 15.0, 2.0).border 
                } else {
                    Rectangle::new_round_border(color::BLACK, 15.0, 1.0).border
                }
            },
        );
        button3.display.rect.color = if value.settings.view.show_contact_points { color::GREEN } else { color::RED };

        Self { 
            components: vec![
                Box::new(button1),
                Box::new(button2),
                Box::new(button3),
            ] 
        }
    }
}


struct Inventory {
    components: Vec<Box<dyn UIComponent>>
}

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

/*
struct GameMenu {
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
*/