use core::f64;
use std::cell::RefCell;
use std::rc::Rc;

use crate::game;
use crate::game::StringStart;
use crate::game::Utility;
use crate::physics::material::CONCRETE;
use crate::physics::material::ICE;
use crate::physics::material::STEEL;
use crate::physics::material::WOOD;
use crate::physics::rigid_body::RigidBody;
use crate::physics::shape::Shape;
use crate::physics::string_body::Attachment;
use crate::physics::string_body::StringBody;
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
use graphics::rectangle::square;
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
    utility_menu: GUI,
    show_utility_menu: bool,
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

        if self.show_utility_menu {
            self.utility_menu.draw(glyphs, c, gl);
        }

        match &game.settings.utility {
            game::Utility::Launch => {
                if let Some(target) = game.projectile.target {
                    let projectile_pos = game.projectile.body.shape.get_center();
                    let line = [projectile_pos.x, projectile_pos.y, target.x, target.y];
                    graphics::line(color::BLACK, 1.0, line, game.camera_transform, gl);

                    let projectile = RigidBody::new(game.projectile.body.shape.scale(game.projectile.scale), game.projectile.body.material, false);
                    projectile.draw(game.camera_transform, game.textures.get(&projectile.material.name).unwrap(), c, gl);
                }
            }
            game::Utility::String(string_start) => {
                if let Some(start) = string_start {
                    let square = square(start.position.x, start.position.y, 5.0);
                    ellipse(color:: RED, square, game.camera_transform, gl);
                }
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
                        "utility" => self.show_utility_menu = !self.show_utility_menu,
                        "physics" => self.show_physics_menu = !self.show_physics_menu,                        
                        _ => {}
                    }
                },
                GUIEvent::StateChange(state) => next_state = Some(state),
                GUIEvent::Quit => std::process::exit(0),
                _ => {}
            }
        }

        let mut update_menu = |menu: &mut GUI, hide_on_click: bool| -> bool {
            let mut show_menu = true;
            for component in menu.components.as_mut_slice() {
                let event = component.update(cursor_pos, e, game);
                if !matches!(event, GUIEvent::None) {
                    interaction = true;
                }
                match event {
                    GUIEvent::Click => if hide_on_click {
                        show_menu = false;
                    }
                    _ => {}
                }
            }
            show_menu
        };

        if self.show_shape_menu {
            self.show_shape_menu = update_menu(&mut self.shape_menu, true);
        }

        if self.show_material_menu {
            self.show_material_menu = update_menu(&mut self.material_menu, true);
        }

        if self.show_utility_menu {
            self.show_utility_menu = update_menu(&mut self.utility_menu, true);
        }

        if self.show_physics_menu {
            self.show_physics_menu = update_menu(&mut self.physics_menu, false);
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
            match &mut game.settings.utility {
                game::Utility::Launch => if !interaction {
                    game.projectile.target = Some(cursor_world_position.into())
                },
                game::Utility::String(string_start) => if !interaction {
                    if let Some(start) = string_start {
                        let mut end_position = cursor_world_position;
                        let mut end_attachment = None;
                        for obj_ref in game.bodies.as_slice() {
                            let obj = obj_ref.borrow();
                            if obj.shape.contains_point(end_position) {
                                end_position = if obj.is_static {
                                    obj.shape.find_closest_surface_point(end_position).0
                                } else {
                                    obj.shape.get_center()
                                };
                                let rel_pos = (end_position - obj.shape.get_center()).rotate(-obj.shape.get_rotation());
                                end_attachment = Some(Attachment { obj_ref: obj_ref.clone(), rel_pos });
                                break;
                            } 
                        }

                        if let Some(att) = &start.attachment {
                            start.position = att.get_attachment_point();
                        }
                        let len = (end_position - start.position).len();
                        let num_joints  = len as usize / 10;
                        if num_joints > 1 {
                            let mut string = StringBody::new(start.position, end_position, num_joints);
                            string.joints[0].attachment = start.attachment.clone();
                            string.joints.last_mut().unwrap().attachment = end_attachment;
                            game.strings.push(Rc::new(RefCell::new(string)));
                        }
                        
                        start.attachment = None;
                        *string_start = None;
                    } else {
                        let mut start_position = cursor_world_position;
                        let mut start_attachment = None;
                        for obj_ref in game.bodies.as_slice() {
                            let obj = obj_ref.borrow();
                            if obj.shape.contains_point(start_position) {
                                start_position = if obj.is_static {
                                    obj.shape.find_closest_surface_point(start_position).0
                                } else {
                                    obj.shape.get_center()
                                };
                                let rel_pos = (start_position - obj.shape.get_center()).rotate(-obj.shape.get_rotation());
                                start_attachment = Some(Attachment { obj_ref: obj_ref.clone(), rel_pos });
                                break;
                            }
                        }
                        *string_start = Some(StringStart { position: start_position, attachment: start_attachment })
                    }
                }
            }
        }

        // Launch on release
        if let Some(_) = game.projectile.target {
            game.projectile.body.shape.set_center(cursor_world_position);
        }

        if let Some(Button::Mouse(MouseButton::Left)) = e.release_args() {
            match &game.settings.utility {
                game::Utility::Launch => {
                    if let Some(target) = game.projectile.target {
                        let velocity = (target - cursor_world_position) * 2.0;
                        let shape = game.projectile.body.shape.scale(game.projectile.scale);
                        let mut body = RigidBody::new(shape, game.projectile.body.material, false);
                        body.linear_velocity = velocity;
                        game.bodies.push(Rc::new(RefCell::new(body)));
                        game.projectile.target = None;                            
                    }
                },
                game::Utility::String(_) => {}
            }
        }

        let mut player = game.player.borrow_mut();
        if let Some(Button::Keyboard(key)) = e.press_args() {
            match key {
                Key::A => player.linear_velocity.x -= 10.0,
                Key::D => player.linear_velocity.x += 10.0,
                Key::Space => player.linear_velocity.y -= 200.0,  
                Key::Escape => next_state = Some(Box::new(PauseState::from(&*game))),
                _ => {}
            }
        }

        next_state
    }
}

impl From<&Game> for PlayingState {
    fn from(value: &Game) -> Self {
        let dimensions: Vector2f<f64> = [1280.0, 720.0].into();
        let mut gravity_slider = GUISlider2D::new(Vector2f::new(1055.0, 100.0), 200.0, |value, event, game| {
            match event {
                GUIEvent::Change => game.physics.gravity = value * 500.0,
                _ => {}
            }
            event
        });
        gravity_slider.value = value.physics.gravity / 500.0;

        let rect = Rectangle::new_round_border(color::BLACK, 5.0, 1.0);
        let text = Text::new(20);
        let text_box = Display::new(rect, DisplayContent::Text(text, "G".to_string()));
        let gravity_display = GUIButton::new(
            Vector2f::new(1055.0, 25.0), 
            Vector2f::new(200.0, 50.0), 
            text_box,
            |btn, event, game| {
                if let DisplayContent::Text(_, str) = &mut btn.display.content {
                    *str = format!("G: {:.2} m/s²", game.physics.gravity.len() / 100.0);
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

        let slot_size = Vector2f::new(90.0, 90.0);

        let utility_display = Display::new(
            Rectangle::new_round_border(color::BLACK, 5.0, 1.0),
            DisplayContent::Text(Text::new(20), "L".to_string()),
        );
        let utility_button = GUIButton::new(
            Vector2f::new(25.0, dimensions.y - 125.0), 
            slot_size, 
            utility_display, 
            |btn, event, game| {
                match event {
                    GUIEvent::Hover => btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 15.0, 2.0).border,
                    GUIEvent::UnHover => btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 15.0, 1.0).border,
                    GUIEvent::Click => return GUIEvent::Custom("utility".to_string()),
                    _ => {
                        btn.display.content = match game.settings.utility {
                            Utility::Launch => DisplayContent::Text(Text::new(20), "L".to_string()),
                            Utility::String(_) => DisplayContent::Text(Text::new(20), "S".to_string()),
                        }
                    }
                }
                event
            }
        );

        let launch_button = GUIButton::new(
            Vector2f::new(25.0, dimensions.y - 225.0), 
            slot_size, 
            Display::new(
                Rectangle::new_round_border(color::BLACK, 5.0, 1.0),
                DisplayContent::Text(Text::new(20), "L".to_string()), 
            ), 
            |btn, event, game| {
                match event {
                    GUIEvent::Hover => btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 15.0, 2.0).border,
                    GUIEvent::UnHover => btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 15.0, 1.0).border,
                    GUIEvent::Click => game.settings.utility = Utility::Launch,
                    _ => {}
                }
                event
            }
        );

        let string_button = GUIButton::new(
            Vector2f::new(125.0, dimensions.y - 225.0), 
            slot_size, 
            Display::new(
                Rectangle::new_round_border(color::BLACK, 5.0, 1.0),
                DisplayContent::Text(Text::new(20), "S".to_string()), 
            ), 
            |btn, event, game| {
                match event {
                    GUIEvent::Hover => btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 15.0, 2.0).border,
                    GUIEvent::UnHover => btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 15.0, 1.0).border,
                    GUIEvent::Click => game.settings.utility = Utility::String(None),
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
        let shape_slot1 = GUIButton::new(
            Vector2f::new(25.0, 125.0), 
            slot_size,
            shape_display, 
            |btn, event, game| {
                match event {
                    GUIEvent::Hover => btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 15.0, 2.0).border,
                    GUIEvent::UnHover => btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 15.0, 1.0).border,
                    GUIEvent::Click => {
                        if let DisplayContent::Shape(s) = &btn.display.content {
                            game.projectile.body.shape = s.clone();
                        }
                        return GUIEvent::Click;
                    }
                    _ => {}
                }
                event
            }, 
        );

        let mut shape_slot2 = shape_slot1.clone();
        shape_slot2.position.x += 100.0;
        let shape2 = Polygon::new_rectangle(Vector2f::zero(), 55.0, 25.0, color::BLACK);
        shape_slot2.display.content = DisplayContent::Shape(ShapeType::Polygon(shape2));

        let mut shape_slot3 = shape_slot2.clone();
        shape_slot3.position.x += 100.0;
        let verts = vec![
            Vector2f::new(-20.0, 20.0),
            Vector2f::new(-20.0, 0.0),
            Vector2f::new(0.0, -20.0),
            Vector2f::new(20.0, 0.0),
            Vector2f::new(20.0, 20.0) 
        ];
        let shape3 = Polygon::new(verts, Vector2f::zero(), color::BLACK);
        shape_slot3.display.content = DisplayContent::Shape(ShapeType::Polygon(shape3));

        let mut shape_slot4 = shape_slot3.clone();
        shape_slot4.position.x += 100.0;
        let verts = vec![
            Vector2f::new(0.0, -28.0),
            Vector2f::new(15.0, 0.0),
            Vector2f::new(0.0, 28.0),
            Vector2f::new(-15.0, 0.0) 
        ];
        let shape4 = Polygon::new(verts, Vector2f::zero(), color::BLACK);
        shape_slot4.display.content = DisplayContent::Shape(ShapeType::Polygon(shape4));

        let rect = Rectangle::new_round_border(color::BLACK, 5.0, 1.0);
        let shape_button = GUIButton::new(
            Vector2f::new(25.0, 25.0), 
            slot_size, 
            Display::new(rect, DisplayContent::Body(
                value.projectile.body.scale(value.projectile.scale), 
                value.textures.get(&value.projectile.body.material.name).unwrap().clone())),
            |btn, event, game| {
                match event {
                    GUIEvent::Click => return GUIEvent::Custom("shape".to_string()),
                    GUIEvent::Hover => { btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 5.0, 2.0).border },
                    GUIEvent::UnHover => { btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 5.0, 1.0).border },
                    _ => {
                        btn.display.content = DisplayContent::Body(
                            game.projectile.body.scale(game.projectile.scale), 
                            game.textures.get(&game.projectile.body.material.name).unwrap().clone(),
                        )
                    },
                }
                event
            }
        );

        let material_button = GUIButton::new(
            Vector2f::new(150.0, 25.0), 
            slot_size, 
            Display::new(rect, DisplayContent::Text(Text::new(0), String::new())),
            |btn, event, game| {
                match event {
                    GUIEvent::Click => return GUIEvent::Custom("material".to_string()),
                    GUIEvent::Hover => { btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 5.0, 2.0).border },
                    GUIEvent::UnHover => { btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 5.0, 1.0).border },
                    _ => btn.display.content = DisplayContent::Image(game.textures.get(&game.projectile.body.material.name).unwrap().clone()),
                }
                event
            }
        );
   
        let concrete_slot = GUIButton::new(
            Vector2f::new(25.0, 225.0), 
            slot_size, 
            Display::new(rect, DisplayContent::Image(value.textures.get(&material::MaterialName::Concrete).unwrap().clone())),
            |btn, event, game| {
                match event {
                    GUIEvent::Click => game.projectile.body.material = CONCRETE,
                    GUIEvent::Hover => { btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 5.0, 2.0).border },
                    GUIEvent::UnHover => { btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 5.0, 1.0).border },
                    _ => {}
                }
                event
            }
        );

        let ice_slot = GUIButton::new(
            Vector2f::new(125.0, 225.0), 
            slot_size, 
            Display::new(rect, DisplayContent::Image(value.textures.get(&material::MaterialName::Ice).unwrap().clone())),
            |btn, event, game| {
                match event {
                    GUIEvent::Click => game.projectile.body.material = ICE,
                    GUIEvent::Hover => { btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 5.0, 2.0).border },
                    GUIEvent::UnHover => { btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 5.0, 1.0).border },
                    _ => {}
                }
                event
            }
        );

        let wood_slot = GUIButton::new(
            Vector2f::new(225.0, 225.0), 
            slot_size, 
            Display::new(rect, DisplayContent::Image(value.textures.get(&material::MaterialName::Wood).unwrap().clone())),
            |btn, event, game| {
                match event {
                    GUIEvent::Click => game.projectile.body.material = WOOD,
                    GUIEvent::Hover => { btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 5.0, 2.0).border },
                    GUIEvent::UnHover => { btn.display.rect.border = Rectangle::new_round_border(color::BLACK, 5.0, 1.0).border },
                    _ => {}
                }
                event
            }
        );

        let steel_slot = GUIButton::new(
            Vector2f::new(325.0, 225.0), 
            slot_size, 
            Display::new(rect, DisplayContent::Image(value.textures.get(&material::MaterialName::Steel).unwrap().clone())),
            |btn, event, game| {
                match event {
                    GUIEvent::Click => game.projectile.body.material = STEEL,
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
                    GUIEvent::Change => game.projectile.scale = (value + 0.25) * 4.0 / 3.0,
                    _ => {} 
                }
                event
            }
        );
        scale.value = (value.projectile.scale * 3.0 / 4.0) - 0.25;

        Self { 
            gui: GUI { components: vec![Box::new(gravity_display), Box::new(scale), Box::new(shape_button), Box::new(material_button), Box::new(utility_button)] }, 
            shape_menu: GUI { components: vec![Box::new(shape_slot1), Box::new(shape_slot2), Box::new(shape_slot3), Box::new(shape_slot4)] }, 
            show_shape_menu: false,
            material_menu: GUI { components: vec![Box::new(concrete_slot), Box::new(ice_slot), Box::new(wood_slot), Box::new(steel_slot)] },
            show_material_menu: false,
            physics_menu: GUI { components: vec![Box::new(gravity_slider)] },
            show_physics_menu: false,
            utility_menu: GUI { components: vec![Box::new(launch_button), Box::new(string_button)] },
            show_utility_menu: false,
        }   
    }
}