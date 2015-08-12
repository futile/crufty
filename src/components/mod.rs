use std::collections::HashSet;

use nc::bounding_volume::{AABB2};

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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CollisionType {
    Solid,
    Trigger,
}

#[derive(Clone)]
pub struct Collision {
    coll_type: CollisionType,
    width: f32,
    height: f32
}

impl Collision {
    pub fn new(width: f32, height: f32,collision_type: CollisionType) -> Collision {
        Collision {
            coll_type: collision_type,
            width: width,
            height: height
        }
    }

    pub fn collision_type(&self) -> CollisionType {
        self.coll_type
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn height(&self) -> f32 {
        self.height
    }
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
