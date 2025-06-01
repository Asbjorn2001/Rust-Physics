#[derive(Clone, Copy)]
pub struct Material {
    pub name: MaterialName,
    pub density:f64, // g/cm³
    pub restitution: f64,
    pub static_friction: f64,
    pub dynamic_friction: f64,
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum MaterialName {
    Rubber,
    Plastic,
    Concrete,
    Steel,
    Ice,
    Glass,
    Wood,
    Copper,
    Aluminium,
    Dirt,
    HumanBody,
}

#[allow(dead_code)]
pub const RUBBER: Material = Material {
    name: MaterialName::Rubber,
    density: 1.1,
    restitution: 0.85,
    static_friction: 0.5,
    dynamic_friction: 0.4,
};

#[allow(dead_code)]
pub const PLASTIC: Material = Material {
    name: MaterialName::Plastic,
    density: 1.175,
    restitution: 0.7,
    static_friction: 0.35,
    dynamic_friction: 0.25,
};

#[allow(dead_code)]
pub const GLASS: Material = Material {
    name: MaterialName::Glass,
    density: 2.5,
    restitution: 0.6,
    static_friction: 0.9,
    dynamic_friction: 0.4,
};

pub const WOOD: Material = Material {
    name: MaterialName::Wood,
    density: 0.7,
    restitution: 0.5,
    static_friction: 0.5,
    dynamic_friction: 0.4,
};

pub const CONCRETE: Material = Material {
    name: MaterialName::Concrete,
    density: 2.4,
    restitution: 0.4,
    static_friction: 0.95,
    dynamic_friction: 0.85,
};

pub const STEEL: Material = Material {
    name: MaterialName::Steel,
    density: 7.85,
    restitution: 0.3,
    static_friction: 0.6,
    dynamic_friction: 0.45,
};

#[allow(dead_code)]
pub const COPPER: Material = Material {
    name: MaterialName::Copper,
    density: 8.94,
    restitution: 0.3,
    static_friction: 0.53,
    dynamic_friction: 0.4,
};

#[allow(dead_code)]
pub const ALUMINIUM: Material = Material {
    name: MaterialName::Aluminium,
    density: 2.7,
    restitution: 0.4,
    static_friction: 0.6,
    dynamic_friction: 0.47,
};

#[allow(dead_code)]
pub const DIRT: Material = Material {
    name: MaterialName::Dirt,
    density: 1.6,
    restitution: 0.1,
    static_friction: 0.55,
    dynamic_friction: 0.45,
};

#[allow(dead_code)]
pub const HUMAN_BODY: Material = Material {
    name: MaterialName::HumanBody,
    density: 0.985,
    restitution: 0.3,
    static_friction: 0.6,
    dynamic_friction: 0.42,
};

pub const ICE: Material = Material {
    name: MaterialName::Ice,
    density: 0.917,
    restitution: 0.7,
    static_friction: 0.15,
    dynamic_friction: 0.05,
};

