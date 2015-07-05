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

components! {
    struct LevelComponents {
        #[hot] position: Position,
        #[hot] sprite_info: SpriteInfo,
    }
}
