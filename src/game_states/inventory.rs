use crate::physics::circle::Circle;
use crate::physics::polygon::Polygon;
use crate::physics::shape_type::ShapeType;
use crate::game_states::components::*;
use crate::Vector2f;
use crate::Game;
use crate::piston::*;
use crate::color;
use crate::graphics::*;
use crate::opengl_graphics::*;
use crate::GlyphCache;
use crate::game_states::GameState;
use crate::game_states::playing::Playing;


pub struct Inventory {
    components: Vec<Box<dyn UIComponent>>
}

impl GameState for Inventory {
    fn draw(&self, game: &Game, glyphs: &mut GlyphCache<'static, (), Texture>, c: Context, gl: &mut GlGraphics) {
        game.draw(glyphs, c, gl);
        
        for component in self.components.as_slice() {
            component.draw(glyphs, c, gl);
        }

        let rect = Rectangle::new_round_border(color::RED, 5.0, 2.0);
        let display = Display::new(rect, DisplayContent::Shape(game.projectile.scale(game.projectile_scale)));
        display.draw(Vector2f::new(590.0, 200.0), Vector2f::new(100.0, 100.0), glyphs, c, gl);
    }

    fn update(&mut self, cursor_pos: Vector2f<f64>, e: &Event, game: &mut Game) -> Option<Box<dyn GameState>> {
        let mut next_state = None;
        if let Some(args) = e.update_args() {
            game.update(&args);
        }

        for component in self.components.as_mut_slice() {
            match component.update(cursor_pos, e, game) {
                UIEvent::StateChange(state) => next_state = Some(state),
                _ => {}
            } 
        }

        if let Some(Button::Keyboard(key)) = e.press_args() {
            match key {
                Key::Escape | Key::E => next_state = Some(Box::new(Playing::from(&*game))),
                _ => {}
            }
        }

        next_state
    }
}


impl From<&Game> for Inventory {
    fn from(value: &Game) -> Self {
        let mut rect = Rectangle::new_round_border(color::BLACK, 5.0, 1.0);
        rect.color = color::GRAY;

        let shape1 = Circle::new(Vector2f::zero(), 25.0, color::RED);
        let shape_display = Display::new(rect, DisplayContent::Shape(ShapeType::Circle(shape1)));

        let slot1 = UIButton::new(
            Vector2f::new(100.0, 100.0), 
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
            Vector2f::new(0.0, -30.0),
            Vector2f::new(15.0, 0.0),
            Vector2f::new(0.0, 30.0),
            Vector2f::new(-15.0, 0.0) 
        ];
        let shape4 = Polygon::new(
            verts,
            Vector2f::zero(), 
            color::GREEN
        );
        slot4.display.content = DisplayContent::Shape(ShapeType::Polygon(shape4));


        let mut scale = UISlider::new(
            Vector2f::new(800.0, 200.0), 
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
            components: vec![
                Box::new(slot1),
                Box::new(slot2),
                Box::new(slot3),
                Box::new(slot4),
                Box::new(scale),
            ] 
        }
    }
}