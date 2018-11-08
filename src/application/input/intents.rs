#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum InputIntent {
    PrintDebugMessage,
    MoveLeft,
    MoveRight,
    Jump,
    Interact,
}
