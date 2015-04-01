#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

components! {
    LevelComponents {
        #[hot] position: Position
    }
}
