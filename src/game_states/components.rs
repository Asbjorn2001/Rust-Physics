use crate::physics::shape::Renderable;
use crate::physics::shape_type::ShapeType;
use crate::GlyphCache;
use crate::Texture;
use crate::GlGraphics;
use crate::color;
use crate::Vector2f;
use crate::game_states::GameState;
use crate::CharacterCache;
use piston_window::*;
use crate::game::Game;

 pub enum UIEvent {
    None,
    Click,
    Change,
    Hover,
    UnHover,
    Custom(String),
    StateChange(Box<dyn GameState>),
    Quit,
 }

pub trait UIComponent {
    fn draw(&self, glyphs: &mut GlyphCache<'static, (), Texture>, c: Context, gl: &mut GlGraphics);

    fn update(&mut self, cursor_pos: Vector2f<f64>, e: &Event, game: &mut Game) -> UIEvent;
}

pub struct UISlider2D {
    position: Vector2f<f64>,
    size: f64,
    pub value: Vector2f<f64>,
    pressed: bool,
    on_update: fn(Vector2f<f64>, UIEvent, &mut Game) -> UIEvent,
}

impl UISlider2D {
    pub fn new(position: Vector2f<f64>, size: f64, on_update: fn(Vector2f<f64>, UIEvent, &mut Game) -> UIEvent) -> Self {
        Self { 
            position, 
            size, 
            value: Vector2f::new(0.0, 0.0), 
            pressed: false, 
            on_update, 
        }
    }

    pub fn contains_cursor(&self, cursor_pos: Vector2f<f64>) -> bool {
        let radius = self.size / 2.0;
        let center = self.position + Vector2f::new(radius, radius);

        (cursor_pos - center).len() <= radius
    }
}

impl UIComponent for UISlider2D {
    fn draw(&self, glyphs: &mut GlyphCache<'static, (), Texture>, c: Context, gl: &mut GlGraphics) {
        let transform = c.transform.trans_pos(self.position);
        let rect = [0.0, 0.0, self.size, self.size];
        graphics::ellipse(color::SILVER, rect, transform, gl);

        let radius = self.size / 2.0;
        let knob_center = Vector2f::new(radius, radius) + self.value * radius;
        let line = [radius, radius, knob_center.x, knob_center.y];
        graphics::line(color::RED, 2.0, line, transform, gl);

        let border = graphics::Ellipse::new_border(color::BLACK, 1.0);
        border.draw(rect, &c.draw_state, transform, gl);
    }

    fn update(&mut self, cursor_pos: Vector2f<f64>, e: &Event, game: &mut Game) -> UIEvent {
        if let Some(Button::Mouse(MouseButton::Left)) = e.press_args() {
            if self.contains_cursor(cursor_pos) {
                self.pressed = true;
            }
        } else if let Some(Button::Mouse(MouseButton::Left)) = e.release_args() {
            self.pressed = false;
        }

        if self.pressed {
            let radius = self.size / 2.0;
            let center = self.position + Vector2f::new(radius, radius);
            self.value = (cursor_pos - center) / radius;
            if self.value.len() > 1.0 {
                self.value = self.value.normalize();
            }
            return (self.on_update)(self.value, UIEvent::Change, game);
        }

        (self.on_update)(self.value, UIEvent::None, game)
    }
}

pub struct UISlider {
    position: Vector2f<f64>,
    size: Vector2f<f64>,
    pub value: f64,
    color: [f32; 4],
    pressed: bool,
    on_update: fn(f64, UIEvent, &mut Game) -> UIEvent,
}

impl UISlider {
    pub fn new(position: Vector2f<f64>, size: Vector2f<f64>, color: [f32; 4], on_update: fn(f64, UIEvent, &mut Game) -> UIEvent) -> Self {
        Self { 
            position, 
            size, 
            value: 0.5, 
            color, 
            pressed: false,
            on_update,
        }
    }

    fn contains_cursor(&self, cursor_pos: Vector2f<f64>) -> bool {
        let top_left = self.position;
        let bottom_right = self.position + self.size;

        let knob_center_x = self.position.x + self.value * self.size.x;
        let knob_center_y = self.position.y + self.size.y / 2.0;
        let knob_radius = self.size.y;

        (cursor_pos - Vector2f::new(knob_center_x, knob_center_y)).len() <= knob_radius ||
        cursor_pos.x >= top_left.x && 
        cursor_pos.x <= bottom_right.x &&
        cursor_pos.y >= top_left.y &&
        cursor_pos.y <= bottom_right.y
    }
}

impl UIComponent for UISlider {
    fn draw(&self, glyphs: &mut GlyphCache<'static, (), Texture>, c: Context, gl: &mut GlGraphics) {
        let transform = c.transform.trans_pos(self.position);
        let rect = [0.0, 0.0, self.size.x, self.size.y];
        
        Rectangle::new_round(color::SILVER, 5.0)
        .border(rectangle::Border { color: color::BLACK, radius: 1.0 })
        .draw(rect, &c.draw_state, transform, gl);

        let knob_x = self.value * self.size.x - self.size.y;
        let circle = rectangle::square(knob_x, -self.size.y / 2.0, self.size.y * 2.0);
        Ellipse::new(self.color)
        .border(ellipse::Border { color: color::BLACK, radius: 1.0 })
        .draw(circle, &c.draw_state, transform, gl);
    }

    fn update(&mut self, cursor_pos: Vector2f<f64>, e: &Event, game: &mut Game) -> UIEvent {
        if let Some(Button::Mouse(MouseButton::Left)) = e.press_args() {
            if self.contains_cursor(cursor_pos) {
                self.pressed = true;
            }
        } else if let Some(Button::Mouse(MouseButton::Left)) = e.release_args() {
            self.pressed = false;
        }

        if self.pressed {
            self.value = ((cursor_pos.x - self.position.x) / self.size.x).clamp(0.0, 1.0);
            return (self.on_update)(self.value, UIEvent::Change, game);
        }

        (self.on_update)(self.value, UIEvent::None, game)
    }
}


#[derive(Clone)]
pub struct UIButton {
    pub position: Vector2f<f64>,
    pub size: Vector2f<f64>,
    pub display: Display,
    pub is_hovered: bool,
    pub on_update: fn(&mut Self, UIEvent, &mut Game) -> UIEvent,
}

impl UIButton {
    pub fn new(
        position: Vector2f<f64>, 
        size: Vector2f<f64>, 
        display: Display, 
        on_update: fn(&mut Self, UIEvent, &mut Game) -> UIEvent) -> Self {
        Self { 
            position,
            size,
            display, 
            is_hovered: false,
            on_update,
        }
    }

    fn contains_cursor(&self, cursor_pos: Vector2f<f64>) -> bool {
        let top_left = self.position;
        let bottom_right = self.position + self.size;

        cursor_pos.x >= top_left.x && 
        cursor_pos.x <= bottom_right.x &&
        cursor_pos.y >= top_left.y &&
        cursor_pos.y <= bottom_right.y
    }
}

impl UIComponent for UIButton {
    fn draw(&self, glyphs: &mut GlyphCache<'static, (), Texture>, c: Context, gl: &mut GlGraphics) {
        self.display.draw(self.position, self.size, glyphs, c, gl);
    }

    fn update(&mut self, cursor_pos: Vector2f<f64>, e: &Event, game: &mut Game) -> UIEvent {
        let mut event = UIEvent::None;
        if self.contains_cursor(cursor_pos) {
            if !self.is_hovered { 
                event = UIEvent::Hover 
            }
            self.is_hovered = true;
            if let Some(Button::Mouse(MouseButton::Left)) = e.press_args() {
                event = UIEvent::Click;
            }    
        } else {
            if self.is_hovered {
                event = UIEvent::UnHover;
            }
            self.is_hovered = false;
        }

        (self.on_update)(self, event, game)
    }
}

#[derive(Clone)]
pub enum DisplayContent {
    Text((Text, String)),
    Shape(ShapeType),
}

// A rectangular box with centered text
#[derive(Clone)]
pub struct Display {
    pub rect: Rectangle,
    pub content: DisplayContent,
}


impl Display {
    pub fn new(rect: Rectangle, content: DisplayContent) -> Self {
        Self { 
            rect,
            content,
        }
    }
    
    pub fn draw(&self, position: Vector2f<f64>, size: Vector2f<f64>, glyphs: &mut GlyphCache<'static, (), Texture>, c: Context, gl: &mut GlGraphics) {
        let rect = [0.0, 0.0, size.x, size.y];
        self.rect.draw(rect, &c.draw_state, c.transform.trans_pos(position), gl);

        match &self.content {
            DisplayContent::Text((text, str)) => {
                let text_width = glyphs.width(text.font_size, &str).unwrap_or(0.0);
                let text_x = position.x + (size.x - text_width) / 2.0;
                let text_y = position.y + (size.y + text.font_size as f64) / 2.0;
                text.draw(&str, glyphs, &c.draw_state, c.transform.trans(text_x, text_y), gl).unwrap();
            }
            DisplayContent::Shape(shape) => {
                let offset = (position + size / 2.0) - shape.get_center();
                shape.draw(c.transform.trans_pos(offset), gl);
            } 
        }        
    }
}