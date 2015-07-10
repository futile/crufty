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

components! {
    struct LevelComponents {
        #[hot] position: Position,
        #[hot] sprite_info: SpriteInfo,
        #[cold] camera: Camera,
    }
}
