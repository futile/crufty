use std::collections::HashSet;

use nc::shape::Cuboid2;
use nc::bounding_volume::{AABB2};
use na::{Vec2};

use systems::WorldViewport;
use application::{InputContext, InputIntent};

use util::{TextureInfo};

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

#[derive(Debug, Clone)]
pub struct Collision {
    coll_type: CollisionType,
    r_x: Cuboid2<f32>,
    off_x: Vec2<f32>,
    r_y: Cuboid2<f32>,
    off_y: Vec2<f32>,
}

impl Collision {
    pub fn new_single(rect: Cuboid2<f32>, off: Vec2<f32>, collision_type: CollisionType) -> Collision {
        Collision {
            coll_type: collision_type,
            r_x: rect.clone(),
            off_x: off.clone(),
            r_y: rect,
            off_y: off,
        }
    }

    pub fn new_dual(rect_x: Cuboid2<f32>, off_x: Vec2<f32>, rect_y: Cuboid2<f32>, off_y: Vec2<f32>, collision_type: CollisionType) -> Collision {
        Collision {
            coll_type: collision_type,
            r_x: rect_x,
            off_x: off_x,
            r_y: rect_y,
            off_y: off_y,
        }
    }

    pub fn collision_type(&self) -> CollisionType {
        self.coll_type
    }

    pub fn rect_x(&self) -> &Cuboid2<f32> {
        &self.r_x
    }

    pub fn off_x(&self) -> &Vec2<f32> {
        &self.off_x
    }

    pub fn rect_y(&self) -> &Cuboid2<f32> {
        &self.r_y
    }

    pub fn off_y(&self) -> &Vec2<f32> {
        &self.off_y
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
