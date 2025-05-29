use core::f64;
use std::cell::RefCell;
use std::rc::Rc;

use crate::physics::material::CONCRETE;
use crate::physics::material::ICE;
use crate::physics::material::STEEL;
use crate::physics::material::WOOD;
use crate::physics::rigid_body::RigidBody;
use crate::physics::*;
use crate::game_state::gui_component::*;
use crate::Vector2f;
use crate::color;
use crate::Text;
use crate::GlyphCache;
use crate::game_state::GameState;
use crate::physics::circle::Circle;
use crate::physics::polygon::Polygon;
use crate::physics::shape_type::ShapeType;
use super::pause_state::PauseState;
use crate::Texture;
use graphics::math::translate;
use piston_window::*;
use vecmath::row_mat2x3_transform_pos2;
use crate::game::Game;
use crate::GlGraphics;
use super::gui::GUI;

pub struct PlayingState {
    pub gui: GUI,
    shape_menu: GUI,
    show_shape_menu: bool,
    physics_menu: GUI,
    show_physics_menu: bool,
    material_menu: GUI,
    show_material_menu: bool,
}

impl GameState for PlayingState {
    fn draw(&self, game: &Game, glyphs: &mut GlyphCache<'static, (), Texture>, c: Context, gl: &mut GlGraphics) {
        game.draw(glyphs, c, gl);

        self.gui.draw(glyphs, c, gl);
        
        if self.show_shape_menu {
            self.shape_menu.draw(glyphs, c, gl);
        }

        if self.show_physics_menu {
            self.physics_menu.draw(glyphs, c, gl);
        }

        if self.show_material_menu {
            self.material_menu.draw(glyphs, c, gl);
        }

        if let Some(target) = game.target {
            if game.settings.enable_launch {
                let projectile_pos = game.projectile.shape.get_center();
                let line = [projectile_pos.x, projectile_pos.y, target.x, target.y];
                graphics::line(color::BLACK, 1.0, line, game.camera_transform, gl);

                let projectile = RigidBody::new(game.projectile.shape.scale(game.projectile_scale), game.projectile.material, false);
                projectile.draw(game.camera_transform, game.textures.get(&projectile.material.name).unwrap(), c, gl);
            }
        }
    }   

    fn update(&mut self, cursor_pos: Vector2f<f64>, e: &Event, game: &mut Game) -> Option<Box<dyn GameState>> {
        let mut interaction = false;
        let mut next_state = None;
        for component in self.gui.components.as_mut_slice() {
            let event = component.update(cursor_pos, e, game);
            if !matches!(event, GUIEvent::None) {
                interaction = true;
            }
            match component.update(cursor_pos, e, game) {
                GUIEvent::Custom(event) => {
                    match event.as_str() {
                        "shape" => self.show_shape_menu = !self.show_shape_menu,
                        "material" => self.show_material_menu = !self.show_material_menu,
                        "physics" => self.show_physics_menu = !self.show_physics_menu,                        
                        _ => {}
                    }
                },
                GUIEvent::StateChange(state) => next_state = Some(state),
                GUIEvent::Quit => std::process::exit(0),
                _ => {}
            }
        }

        if self.show_shape_menu {
            for component in self.shape_menu.components.as_mut_slice() {
                let event = component.update(cursor_pos, e, game);
                if !matches!(event, GUIEvent::None) {
                    interaction = true;
                }
                match event {
                    GUIEvent::Click => self.show_shape_menu = false,
                    _ => {}
                }
            }
        }

        if self.show_material_menu {
            for component in self.material_menu.components.as_mut_slice() {
                let event = component.update(cursor_pos, e, game);
                if !matches!(event, GUIEvent::None) {
                    interaction = true;
                }
                match event {
                    GUIEvent::Click => self.show_material_menu = false,
                    _ => {}
                }
            }
        }

        if self.show_physics_menu {
            for component in self.physics_menu.components.as_mut_slice() {
                let event = component.update(cursor_pos, e, game);
                if !matches!(event, GUIEvent::None) {
                    interaction = true;
                }
            }
        }
        
        // Update game logic
        if let Some(args) = e.update_args() {
            game.update(args.dt);
        }

        let dims = Vector2f::from(game.context.get_view_size());
        let inv_scale = 1.0 / game.settings.camera.scale;
        let transform = translate(game.settings.camera.position.into()).scale(inv_scale, inv_scale).trans_pos(-dims / 2.0);
        let cursor_world_position = Vector2f::from(row_mat2x3_transform_pos2(transform, cursor_pos.into()));

        // Set target on press
        if let Some(Button::Mouse(MouseButton::Left)) = e.press_args() {
            if game.settings.enable_launch && !interaction {
                game.target = Some(cursor_world_position.into());
            } 
        }

        // Launch on release
        if let Some(target) = game.target {
            game.projectile.shape.set_center(cursor_world_position);
            if let Some(Button::Mouse(MouseButton::Left)) = e.release_args() {
                let velocity = (target - cursor_world_position) * 2.0;

                let shape = game.projectile.shape.scale(game.projectile_scale);
                let mut body = RigidBody::new(shape, game.projectile.material, false);
                body.linear_velocity = velocity;
                game.bodies.push(Rc::new(RefCell::new(body)));

                game.target = None;
            }
        } else {
            //let first_joint = &mut game.string.joints[0]; 
            //first_joint.velocity = cursor_world_position - first_joint.position;
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
        let mut gravity_slider = GUISlider2D::new(Vector2f::new(1055.0, 100.0), 200.0, |value, event, game| {
            match event {
                GUIEvent::Change => game.settings.physics.gravity = value * 500.0,
                _ => {}
            }
            event
        });
        gravity_slider.value = value.settings.physics.gravity / 500.0;

        let rect = Rectangle::new_round_border(color::BLACK, 5.0, 1.0);
        let text = Text::new(20);
        let text_box = Display::new(rect, DisplayContent::Text((text, "G".to_string())));
        let gravity_display = GUIButton::new(
            Vector2f::new(1055.0, 25.0), 
            Vector2f::new(200.0, 50.0), 
            text_box,
            |btn, event, game| {
                if let DisplayContent::Text(text) = &mut btn.display.content {
                    text.1 = format!("G: {:.2} m/s²", game.settings.physics.gravity.len() / 100.0);
                }
                match event {
                    GUIEvent::Hover => btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 15.0, 2.0).border,
                    GUIEvent::UnHover => btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 15.0, 1.0).border,
                    GUIEvent::Click => return GUIEvent::Custom("physics".to_string()),
                    _ => {}
                }
                event
            } 
        );

        // Shape selection
        let mut rect = Rectangle::new_round_border(color::BLACK, 5.0, 1.0);
        rect.color = color::GRAY;
        let shape1 = Circle::new(Vector2f::zero(), 25.0, color::BLACK);
        let shape_display = Display::new(rect, DisplayContent::Shape(ShapeType::Circle(shape1)));
        let slot1 = GUIButton::new(
            Vector2f::new(25.0, 125.0), 
            Vector2f::new(90.0, 90.0), 
            shape_display, 
            |btn, event, game| {
                match event {
                    GUIEvent::Hover => btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 15.0, 2.0).border,
                    GUIEvent::UnHover => btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 15.0, 1.0).border,
                    GUIEvent::Click => {
                        if let DisplayContent::Shape(s) = &btn.display.content {
                            game.projectile.shape = s.clone();
                        }
                        return GUIEvent::Click;
                    }
                    _ => {}
                }
                event
            }, 
        );

        let mut slot2 = slot1.clone();
        slot2.position.x += 100.0;
        let shape2 = Polygon::new_rectangle(Vector2f::zero(), 55.0, 25.0, color::BLACK);
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
        let shape3 = Polygon::new(verts, Vector2f::zero(), color::BLACK);
        slot3.display.content = DisplayContent::Shape(ShapeType::Polygon(shape3));

        let mut slot4 = slot3.clone();
        slot4.position.x += 100.0;
        let verts = vec![
            Vector2f::new(0.0, -28.0),
            Vector2f::new(15.0, 0.0),
            Vector2f::new(0.0, 28.0),
            Vector2f::new(-15.0, 0.0) 
        ];
        let shape4 = Polygon::new(verts, Vector2f::zero(), color::BLACK);
        slot4.display.content = DisplayContent::Shape(ShapeType::Polygon(shape4));

        let rect = Rectangle::new_round_border(color::BLACK, 5.0, 1.0);
        let shape_display = GUIButton::new(
            Vector2f::new(25.0, 25.0), 
            Vector2f::new(90.0, 90.0), 
            Display::new(rect, DisplayContent::Body(
                value.projectile.scale(value.projectile_scale), 
                value.textures.get(&value.projectile.material.name).unwrap().clone())),
            |btn, event, game| {
                match event {
                    GUIEvent::Click => return GUIEvent::Custom("shape".to_string()),
                    GUIEvent::Hover => { btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 5.0, 2.0).border },
                    GUIEvent::UnHover => { btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 5.0, 1.0).border },
                    _ => {
                        btn.display.content = DisplayContent::Body(
                            game.projectile.scale(game.projectile_scale), 
                            game.textures.get(&game.projectile.material.name).unwrap().clone(),
                        )
                    },
                }
                event
            }
        );

        let material_display = GUIButton::new(
            Vector2f::new(150.0, 25.0), 
            Vector2f::new(90.0, 90.0), 
            Display::new(rect, DisplayContent::Text((Text::new(0), String::new()))),
            |btn, event, game| {
                match event {
                    GUIEvent::Click => return GUIEvent::Custom("material".to_string()),
                    GUIEvent::Hover => { btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 5.0, 2.0).border },
                    GUIEvent::UnHover => { btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 5.0, 1.0).border },
                    _ => btn.display.content = DisplayContent::Image(game.textures.get(&game.projectile.material.name).unwrap().clone()),
                }
                event
            }
        );
   
        let concrete_slot = GUIButton::new(
            Vector2f::new(25.0, 225.0), 
            Vector2f::new(90.0, 90.0), 
            Display::new(rect, DisplayContent::Image(value.textures.get(&material::MaterialName::Concrete).unwrap().clone())),
            |btn, event, game| {
                match event {
                    GUIEvent::Click => game.projectile.material = CONCRETE,
                    GUIEvent::Hover => { btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 5.0, 2.0).border },
                    GUIEvent::UnHover => { btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 5.0, 1.0).border },
                    _ => {}
                }
                event
            }
        );

        let ice_slot = GUIButton::new(
            Vector2f::new(125.0, 225.0), 
            Vector2f::new(90.0, 90.0), 
            Display::new(rect, DisplayContent::Image(value.textures.get(&material::MaterialName::Ice).unwrap().clone())),
            |btn, event, game| {
                match event {
                    GUIEvent::Click => game.projectile.material = ICE,
                    GUIEvent::Hover => { btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 5.0, 2.0).border },
                    GUIEvent::UnHover => { btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 5.0, 1.0).border },
                    _ => {}
                }
                event
            }
        );

        let wood_slot = GUIButton::new(
            Vector2f::new(225.0, 225.0), 
            Vector2f::new(90.0, 90.0), 
            Display::new(rect, DisplayContent::Image(value.textures.get(&material::MaterialName::Wood).unwrap().clone())),
            |btn, event, game| {
                match event {
                    GUIEvent::Click => game.projectile.material = WOOD,
                    GUIEvent::Hover => { btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 5.0, 2.0).border },
                    GUIEvent::UnHover => { btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 5.0, 1.0).border },
                    _ => {}
                }
                event
            }
        );

        let steel_slot = GUIButton::new(
            Vector2f::new(325.0, 225.0), 
            Vector2f::new(90.0, 90.0), 
            Display::new(rect, DisplayContent::Image(value.textures.get(&material::MaterialName::Steel).unwrap().clone())),
            |btn, event, game| {
                match event {
                    GUIEvent::Click => game.projectile.material = STEEL,
                    GUIEvent::Hover => { btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 5.0, 2.0).border },
                    GUIEvent::UnHover => { btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 5.0, 1.0).border },
                    _ => {}
                }
                event
            }
        );

        let mut scale = GUISlider::new(
            Vector2f::new(275.0, 60.0), 
            Vector2f::new(200.0, 20.0), 
            color::RED, 
            |value, event, game| {
                match event {
                    GUIEvent::Change => game.projectile_scale = (value + 0.25) * 4.0 / 3.0,
                    _ => {} 
                }
                event
            }
        );
        scale.value = (value.projectile_scale * 3.0 / 4.0) - 0.25;

        Self { 
            gui: GUI { components: vec![Box::new(gravity_display), Box::new(scale), Box::new(shape_display), Box::new(material_display)] }, 
            shape_menu: GUI { components: vec![Box::new(slot1), Box::new(slot2), Box::new(slot3), Box::new(slot4)] }, 
            show_shape_menu: false,
            material_menu: GUI { components: vec![Box::new(concrete_slot), Box::new(ice_slot), Box::new(wood_slot), Box::new(steel_slot)] },
            show_material_menu: false,
            physics_menu: GUI { components: vec![Box::new(gravity_slider)] },
            show_physics_menu: false,
        }   
    }
}