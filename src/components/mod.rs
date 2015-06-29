#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

components! {
    struct LevelComponents {
        #[hot] position: Position
    }
}
