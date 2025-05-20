use crate::GlyphCache;
use crate::Texture;
use crate::GlGraphics;
use crate::Game;
use crate::Border;
use crate::color;
use crate::piston::*;
use crate::graphics::*;
use crate::Vector2f;

/* 
trait StaticComp {
    
}

trait DynamicComp {
    
}

enum UIComp {
    Static(Box<dyn StaticComp>),
    Dynamic(Box<dyn DynamicComp>)
}
 */

pub trait UIComponent {
    fn draw(&self, glyphs: &mut GlyphCache<'static, (), Texture>, c: Context, gl: &mut GlGraphics);

    // Returns true if the component was interacted with
    fn update(&mut self, cursor_pos: Vector2f<f64>, e: &Event, game: &mut Game) -> bool;
}

pub struct UISlider2D {
    position: Vector2f<f64>,
    size: f64,
    value: Vector2f<f64>,
    pressed: bool,
    on_change: fn(Vector2f<f64>, &mut Game),
}

impl UISlider2D {
    pub fn new(position: Vector2f<f64>, size: f64, on_change: fn(Vector2f<f64>, &mut Game)) -> Self {
        Self { 
            position, 
            size, 
            value: Vector2f::new(0.0, 0.0), 
            pressed: false, 
            on_change, 
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

    fn update(&mut self, cursor_pos: Vector2f<f64>, e: &Event, game: &mut Game) -> bool {
        if let Some(Button::Mouse(MouseButton::Left)) = e.press_args() {
            if self.contains_cursor(cursor_pos) {
                self.pressed = true;
                return true;
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
            (self.on_change)(self.value, game);
            return true;
        }

        false
    }
}


pub struct UISlider {
    position: Vector2f<f64>,
    size: Vector2f<f64>,
    value: f64,
    color: [f32; 4],
    pressed: bool,
    on_change: fn(f64, &mut Game),
}

impl UISlider {
    pub fn new(position: Vector2f<f64>, size: Vector2f<f64>, color: [f32; 4], on_change: fn(f64, &mut Game)) -> Self {
        Self { 
            position, 
            size, 
            value: 0.5, 
            color, 
            pressed: false,
            on_change,
        }
    }

    pub fn contains_cursor(&self, cursor_pos: Vector2f<f64>) -> bool {
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
        .border(Border { color: color::BLACK, radius: 1.0 })
        .draw(rect, &c.draw_state, transform, gl);

        let knob_x = self.value * self.size.x - self.size.y;
        let circle = rectangle::square(knob_x, -self.size.y / 2.0, self.size.y * 2.0);
        graphics::ellipse(self.color, circle, transform, gl);
    }

    fn update(&mut self, cursor_pos: Vector2f<f64>, e: &Event, game: &mut Game) -> bool {
        if let Some(Button::Mouse(MouseButton::Left)) = e.press_args() {
            if self.contains_cursor(cursor_pos) {
                self.pressed = true;
            }
        } else if let Some(Button::Mouse(MouseButton::Left)) = e.release_args() {
            self.pressed = false;
        }

        if self.pressed {
            self.value = ((cursor_pos.x - self.position.x) / self.size.x).clamp(0.0, 1.0);
            (self.on_change)(self.value, game);
            return true;
        }

        false
    }
}

pub struct UIDisplay {
    position: Vector2f<f64>,
    size: Vector2f<f64>,
    pub text_box: TextBox,
    on_update: fn(&mut Self, &Game),
}

impl UIDisplay {
    pub fn new(position: Vector2f<f64>, size: Vector2f<f64>, text_box: TextBox, on_update: fn(&mut Self, &Game)) -> Self {
        Self { 
            position, 
            size, 
            text_box, 
            on_update 
        }
    }
}

impl UIComponent for UIDisplay {
    fn draw(&self, glyphs: &mut GlyphCache<'static, (), Texture>, c: Context, gl: &mut GlGraphics) {
        self.text_box.draw(self.position, self.size, glyphs,  c, gl);
    }

    fn update(&mut self, cursor_pos: Vector2f<f64>, e: &Event, game: &mut Game) -> bool {
        (self.on_update)(self, &game);   
        false
    }
}

#[derive(Clone)]
pub struct UIButton {
    position: Vector2f<f64>,
    size: Vector2f<f64>,
    pub display: TextBox,
    pub is_hovered: bool,
    on_click: fn(&mut Self, &mut Game),
    on_change: fn(&mut Self, &mut Game),
}

impl UIButton {
    pub fn new(
        position: Vector2f<f64>, 
        size: Vector2f<f64>, 
        display: TextBox, 
        on_click: fn(&mut Self, &mut Game), 
        on_change: fn(&mut Self, &mut Game)) -> Self {
        Self { 
            position,
            size,
            display, 
            is_hovered: false,
            on_click,
            on_change,
        }
    }

    pub fn contains_cursor(&self, cursor_pos: Vector2f<f64>) -> bool {
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

    fn update(&mut self, cursor_pos: Vector2f<f64>, e: &Event, game: &mut Game) -> bool {
        let mut change;
        if self.contains_cursor(cursor_pos) {
            change = self.is_hovered == false;
            self.is_hovered = true;
            if let Some(Button::Mouse(MouseButton::Left)) = e.press_args() {
                (self.on_click)(self, game);
                change = true;
            }    
        } else {
            change = self.is_hovered;
            self.is_hovered = false;
        }

        if change {
            (self.on_change)(self, game);
        }

        change
    }
}

// A rectangular box with centered text
#[derive(Clone)]
pub struct TextBox {
    pub rect: Rectangle,
    pub text: Text,
    pub string: String,
}

impl TextBox {
    pub fn new(rect: Rectangle, text: Text, string: String) -> Self {
        Self { 
            rect,
            text, 
            string, 
        }
    }
    
    fn draw(&self, position: Vector2f<f64>, size: Vector2f<f64>, glyphs: &mut GlyphCache<'static, (), Texture>, c: Context, gl: &mut GlGraphics) {
        let rect = [0.0, 0.0, size.x, size.y];
        self.rect.draw(rect, &c.draw_state, c.transform.trans_pos(position), gl);

        let text_width = glyphs.width(self.text.font_size, &self.string).unwrap_or(0.0);
        let text_x = position.x + (size.x - text_width) / 2.0;
        let text_y = position.y + (size.y + self.text.font_size as f64) / 2.0;
        self.text.draw(&self.string, glyphs, &c.draw_state, c.transform.trans(text_x, text_y), gl).unwrap();
    }
}