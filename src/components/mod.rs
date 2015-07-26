use std::collections::HashSet;

use nc::bounding_volume::AABB2;
use nc::shape::Cuboid2;

use systems::WorldViewport;
use application::{InputContext, InputIntent};

use util::TextureInfo;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Velocity {
    pub vx: f32,
    pub vy: f32,

    pub last_pos: Position,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Gravity {
    pub f: f32,
}

impl Gravity {
    pub fn new() -> Gravity {
        Gravity {
            f: 1.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum CollisionShape {
    SingleBox(Cuboid2<f32>),
    TwoBoxes {
        x: Cuboid2<f32>,
        y: Cuboid2<f32>
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CollisionAxis {
    XY,
    X,
    Y,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Collision {
    pub shape: CollisionShape,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SpriteInfo {
    pub width: f32,
    pub height: f32,
    pub texture_info: TextureInfo,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Camera {
    pub world_viewport: WorldViewport,
    pub screen_viewport: AABB2<f32>,
    pub resize_world_to_window: bool,
}

#[derive(Debug)]
pub struct KeyboardInput {
    pub input_context: InputContext,
}

pub type Intents = HashSet<InputIntent>;

impl Camera {
    pub fn new(world_viewport: WorldViewport, screen_viewport: AABB2<f32>, resize_world_to_window: bool) -> Camera {
        Camera {
            world_viewport: world_viewport,
            screen_viewport: screen_viewport,
            resize_world_to_window: resize_world_to_window,
        }
    }

    #[allow(dead_code)]
    pub fn new_empty() -> Camera {
        Camera {
            world_viewport: WorldViewport::new_empty(),
            screen_viewport: AABB2::new_invalid(),
            resize_world_to_window: true,
        }
    }
}

components! {
    struct LevelComponents {
        #[hot] position: Position,
        #[hot] collision: Collision,
        #[hot] sprite_info: SpriteInfo,
        #[cold] velocity: Velocity,
        #[cold] gravity: Gravity,
        #[cold] camera: Camera,
        #[cold] keyboard_input: KeyboardInput,
        #[cold] intents: Intents,
    }
}
