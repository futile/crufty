use glium::{self, glutin};

use crate::util::{State, Transition};

mod gamestate;
mod input;

pub use self::gamestate::GameState;
pub use self::input::{InputManager, InputContext, KeyHandler, InputState, InputIntent};

pub enum AppTransition {
    Startup,
    StartGame(glium::Display),
    Shutdown,
    TerminateApplication,
}

impl Transition for AppTransition {
    fn create_state(self) -> Option<Box<dyn State<AppTransition>>> {
        match self {
            AppTransition::Startup => Some(Box::new(StartupState)),
            AppTransition::StartGame(d) => Some(Box::new(GameState::new(d))),
            AppTransition::Shutdown => Some(Box::new(ShutdownState)),
            AppTransition::TerminateApplication => None,
        }
    }
}

pub struct StartupState;

impl State<AppTransition> for StartupState {
    fn run(self: Box<Self>) -> AppTransition {
        use glium::DisplayBuild;

        let display = glutin::WindowBuilder::new()
            .with_dimensions(800, 600)
            .with_title("crufty".to_string())
            .build_glium()
            .unwrap();

        AppTransition::StartGame(display)
    }
}

pub struct ShutdownState;

impl State<AppTransition> for ShutdownState {
    fn run(self: Box<Self>) -> AppTransition {
        AppTransition::TerminateApplication
    }
}
