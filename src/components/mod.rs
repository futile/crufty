use std::collections::HashSet;

use nc::shape::Cuboid2;
use nc::bounding_volume::{HasBoundingVolume, AABB2};
use na::{self, Iso2, Vec2, Pnt2};

use systems::WorldViewport;
use application::{InputContext, InputIntent};

use util::TextureInfo;

use num::traits::Zero;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Position {
    pub fn as_vec(&self) -> Vec2<f32> {
        Vec2::new(self.x, self.y)
    }

    #[allow(unused)]
    pub fn as_pnt(&self) -> Pnt2<f32> {
        Pnt2::new(self.x, self.y)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Velocity {
    pub vx: f32,
    pub vy: f32,

    pub last_pos: Position,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Movement {
    pub vel: Vec2<f32>,
    pub max_vel: Vec2<f32>,
    pub acc: Vec2<f32>,
}

impl Movement {
    pub fn new(max_vel: Vec2<f32>, acc: Vec2<f32>) -> Movement {
        Movement {
            vel: Vec2::zero(),
            max_vel: max_vel,
            acc: acc,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum JumpState {
    Idle,
    Rising,
    MidairIdle,
}

#[derive(Copy, Clone, Debug)]
pub struct Jump {
    pub state: JumpState,
    pub jump_time_remaining: f32,
}

impl Jump {
    pub fn new() -> Jump {
        Jump {
            state: JumpState::Idle,
            jump_time_remaining: 0.0,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Gravity {
    pub f: f32,
}

impl Gravity {
    pub fn new() -> Gravity {
        Gravity { f: 1.0 }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CollisionType {
    Solid, // Trigger,
}

#[derive(Clone)]
pub struct Collision {
    coll_type: CollisionType,
    r_x: Cuboid2<f32>,
    off_x: Vec2<f32>,
    r_y: Cuboid2<f32>,
    off_y: Vec2<f32>,
}

impl Collision {
    pub fn new_single(rect: Cuboid2<f32>,
                      off: Vec2<f32>,
                      collision_type: CollisionType)
                      -> Collision {
        Self::new_dual(rect.clone(), off.clone(), rect, off, collision_type)
    }

    pub fn new_dual(rect_x: Cuboid2<f32>,
                    off_x: Vec2<f32>,
                    rect_y: Cuboid2<f32>,
                    off_y: Vec2<f32>,
                    collision_type: CollisionType)
                    -> Collision {
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

    // pub fn rect_x(&self) -> &Cuboid2<f32> {
    //     &self.r_x
    // }

    pub fn aabb_x(&self, pos: Vec2<f32>) -> AABB2<f32> {
        self.r_x.bounding_volume(&Iso2::new(pos + self.off_x, na::zero()))
    }

    pub fn off_x(&self) -> &Vec2<f32> {
        &self.off_x
    }

    // pub fn rect_y(&self) -> &Cuboid2<f32> {
    //     &self.r_y
    // }

    pub fn aabb_y(&self, pos: Vec2<f32>) -> AABB2<f32> {
        self.r_y.bounding_volume(&Iso2::new(pos + self.off_y, na::zero()))
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

pub type SpritesheetId = u32;

#[derive(Copy, Clone, Debug)]
pub struct SpritesheetAnimation {
    pub id: SpritesheetId,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Facing {
    Left,
    Right,
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
    pub fn new(world_viewport: WorldViewport,
               screen_viewport: AABB2<f32>,
               resize_world_to_window: bool)
               -> Camera {
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
        #[cold] animation: SpritesheetAnimation,
        #[cold] movement: Movement,
        #[cold] facing: Facing,
        #[cold] jump: Jump,
        #[cold] velocity: Velocity,
        #[cold] gravity: Gravity,
        #[cold] camera: Camera,
        #[cold] keyboard_input: KeyboardInput,
        #[cold] intents: Intents,
    }
}
