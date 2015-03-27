use util::{State};

pub enum AppTransition {
    Startup,
    StartGame,
    Shutdown,
    TerminateApplication
}

pub struct StartupState;

impl State<AppTransition> for StartupState {
    fn run(self: Box<Self>) -> AppTransition {
        AppTransition::StartGame
    }
}

pub struct GameState;

impl State<AppTransition> for GameState {
    fn run(self: Box<Self>) -> AppTransition {
        AppTransition::Shutdown
    }
}

pub struct ShutdownState;

impl State<AppTransition> for ShutdownState {
    fn run(self: Box<Self>) -> AppTransition {
        AppTransition::TerminateApplication
    }
}
