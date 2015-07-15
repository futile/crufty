use nc::bounding_volume::AABB2;

use systems::WorldViewport;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SpriteInfo {
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Camera {
    pub world_viewport: WorldViewport,
    pub screen_viewport: AABB2<f32>,
    pub resize_world_to_window: bool,
}

impl Camera {
    pub fn new(world_viewport: WorldViewport, screen_viewport: AABB2<f32>, resize_world_to_window: bool) -> Camera {
        Camera {
            world_viewport: world_viewport,
            screen_viewport: screen_viewport,
            resize_world_to_window: resize_world_to_window,
        }
    }

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
        #[hot] sprite_info: SpriteInfo,
        #[cold] camera: Camera,
    }
}
