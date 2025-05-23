use crate::physics::rigid_body::RigidBody;
use crate::game_states::components::*;
use crate::Vector2f;
use crate::color;
use crate::Text;
use crate::GlyphCache;
use crate::physics::shape::Renderable;
use crate::game_states::GameState;
use crate::physics::circle::Circle;
use crate::physics::polygon::Polygon;
use crate::physics::shape_type::ShapeType;
use super::pause_state::PauseState;
use crate::Texture;
use piston_window::*;
use crate::game::Game;
use crate::GlGraphics;
use super::gui::GUI;

pub struct PlayingState {
    pub gui: GUI,
    shape_menu: GUI,
    show_shape_menu: bool,
}

impl GameState for PlayingState {
    fn draw(&self, game: &Game, glyphs: &mut GlyphCache<'static, (), Texture>, c: Context, gl: &mut GlGraphics) {
        game.draw(glyphs, c, gl);
        
        self.gui.draw(glyphs, c, gl);
        
        if self.show_shape_menu {
            self.shape_menu.draw(glyphs, c, gl);
        }

        if let Some(target) = game.target {
            if game.settings.enable_launch {
                let projectile_pos = game.projectile.get_center();
                let line = [projectile_pos.x, projectile_pos.y, target.x, target.y];
                graphics::line(game.projectile.get_color(), 1.0, line, c.transform, gl);
                game.projectile.scale(game.projectile_scale).draw(c.transform, gl);
            }
        }
    }   

    fn update(&mut self, cursor_pos: Vector2f<f64>, e: &Event, game: &mut Game) -> Option<Box<dyn GameState>> {
        let mut interaction = false;
        let mut next_state = None;
        for component in self.gui.components.as_mut_slice() {
            let event = component.update(cursor_pos, e, game);
            if !matches!(event, UIEvent::None) {
                interaction = true;
            }
            match component.update(cursor_pos, e, game) {
                UIEvent::Custom(string) => {
                    if string.as_str() == "shape_menu" { 
                        self.show_shape_menu = !self.show_shape_menu;
                    }
                }
                UIEvent::StateChange(state) => next_state = Some(state),
                UIEvent::Quit => std::process::exit(0),
                _ => {}
            }
        }

        if self.show_shape_menu {
            for component in self.shape_menu.components.as_mut_slice() {
                let event = component.update(cursor_pos, e, game);
                if !matches!(event, UIEvent::None) {
                    interaction = true;
                }
                match event {
                    UIEvent::Click => self.show_shape_menu = false,
                    _ => {}
                }
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
                Key::Escape => next_state = Some(Box::new(PauseState::from(&*game))),
                _ => {}
            }
        }

        if next_state.is_some() {
            game.target = None;
        }

        next_state
    }
}

impl From<&Game> for PlayingState {
    fn from(value: &Game) -> Self {
        let mut gravity_slider = UISlider2D::new(Vector2f::new(25.0, 425.0), 200.0, |value, event, game| {
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
            Vector2f::new(25.0, 720.0 - 75.0), 
            Vector2f::new(200.0, 50.0), 
            text_box,
            |btn, _, game| {
                if let DisplayContent::Text(text) = &mut btn.display.content {
                    text.1 = format!("G: {:.2} m/s²", game.settings.physics.gravity.len() / 100.0);
                }
                UIEvent::None
            } 
        );

        // Shape selection
        let mut rect = Rectangle::new_round_border(color::BLACK, 5.0, 1.0);
        rect.color = color::GRAY;

        let shape1 = Circle::new(Vector2f::zero(), 25.0, color::RED);
        let shape_display = Display::new(rect, DisplayContent::Shape(ShapeType::Circle(shape1)));

        let slot1 = UIButton::new(
            Vector2f::new(25.0, 125.0), 
            Vector2f::new(90.0, 90.0), 
            shape_display, 
            |btn, event, game| {
                match event {
                    UIEvent::Hover => btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 15.0, 2.0).border,
                    UIEvent::UnHover => btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 15.0, 1.0).border,
                    UIEvent::Click => {
                        if let DisplayContent::Shape(s) = &btn.display.content {
                            game.projectile = s.clone();
                        }
                        return UIEvent::Click;
                    }
                    _ => {}
                }
                event
            }, 
        );

        let mut slot2 = slot1.clone();
        slot2.position.x += 100.0;
        let shape2 = Polygon::new_rectangle(Vector2f::zero(), 55.0, 25.0, color::NAVY);
        slot2.display.content = DisplayContent::Shape(ShapeType::Polygon(shape2));

        let mut slot3 = slot2.clone();
        slot3.position.x += 100.0;
        let verts = vec![
            Vector2f::new(-20.0, 20.0),
            Vector2f::new(-20.0, 0.0),
            Vector2f::new(0.0, -20.0),
            Vector2f::new(20.0, 0.0),
            Vector2f::new(20.0, 20.0) 
        ];
        let shape3 = Polygon::new(
            verts,
            Vector2f::zero(), 
            color::PURPLE
        );
        slot3.display.content = DisplayContent::Shape(ShapeType::Polygon(shape3));

        let mut slot4 = slot3.clone();
        slot4.position.x += 100.0;
        let verts = vec![
            Vector2f::new(0.0, -28.0),
            Vector2f::new(15.0, 0.0),
            Vector2f::new(0.0, 28.0),
            Vector2f::new(-15.0, 0.0) 
        ];
        let shape4 = Polygon::new(
            verts,
            Vector2f::zero(), 
            color::GREEN
        );
        slot4.display.content = DisplayContent::Shape(ShapeType::Polygon(shape4));

        let rect = Rectangle::new_round_border(color::BLACK, 5.0, 1.0);
        let shape_display = UIButton::new(
            Vector2f::new(25.0, 25.0), 
            Vector2f::new(90.0, 90.0), 
            Display::new(rect, DisplayContent::Shape(value.projectile.scale(value.projectile_scale))), 
            |btn, event, game| {
                match event {
                    UIEvent::Click => return UIEvent::Custom("shape_menu".to_string()),
                    UIEvent::Hover => { btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 5.0, 2.0).border },
                    UIEvent::UnHover => { btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 5.0, 1.0).border },
                    _ => { btn.display.content = DisplayContent::Shape(game.projectile.scale(game.projectile_scale)) },
                }
                event
            }
        );

        let mut scale = UISlider::new(
            Vector2f::new(150.0, 60.0), 
            Vector2f::new(200.0, 20.0), 
            color::RED, 
            |value, event, game| {
                match event {
                    UIEvent::Change => game.projectile_scale = (value + 0.25) * 4.0 / 3.0,
                    _ => {} 
                }
                event
            }
        );
        scale.value = (value.projectile_scale * 3.0 / 4.0) - 0.25;

        Self { 
            gui: GUI { components: vec![Box::new(gravity_slider), Box::new(gravity_display), Box::new(scale), Box::new(shape_display)] }, 
            shape_menu: GUI { components: vec![Box::new(slot1), Box::new(slot2), Box::new(slot3), Box::new(slot4)] }, 
            show_shape_menu: false,
        }   
    }
}