pub mod game_controller;
pub mod game_view;

use crate::RigidBody;
use crate::Vector2f;

pub struct Game {
    pub settings: GameSettings,
    pub bodies: Vec<RigidBody>,
    pub target: Option<Vector2f<f64>>,
    pub projectile: RigidBody,
    pub enable_launch: bool,
    pub contacts: Vec<Vector2f<f64>>,
}

pub struct GameSettings {
    pub physics: PhysicsSettings,
    pub view: ViewSettings,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self { 
            physics: PhysicsSettings::default(), 
            view: ViewSettings::default() 
        }
    }
}

pub struct ViewSettings {
    pub show_velocites: bool,
    pub show_contact_points: bool,
}

impl Default for ViewSettings {
    fn default() -> Self {
        Self { 
            show_velocites: true, 
            show_contact_points: true 
        }
    }
}

pub struct PhysicsSettings {
    pub gravity: Vector2f<f64>,
    pub air_density: f64,
}

impl Default for PhysicsSettings {
    fn default() -> Self {
        PhysicsSettings { gravity: Vector2f { x: 0.0, y: 100.0 }, air_density: 0.08 }
    }
}