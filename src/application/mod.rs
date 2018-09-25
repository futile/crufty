use glium::{self, glutin};

use crate::util::{State, Transition};

mod gamestate;
mod input;

pub use self::gamestate::GameState;
pub use self::input::{InputContext, InputIntent, InputManager, InputState, KeyHandler};

pub enum AppTransition {
    Startup,
    StartGame(glium::Display, glutin::EventsLoop),
    Shutdown,
    TerminateApplication,
}

impl Transition for AppTransition {
    fn create_state(self) -> Option<Box<dyn State<AppTransition>>> {
        match self {
            AppTransition::Startup => Some(Box::new(StartupState)),
            AppTransition::StartGame(d, el) => Some(Box::new(GameState::new(d, el))),
            AppTransition::Shutdown => Some(Box::new(ShutdownState)),
            AppTransition::TerminateApplication => None,
        }
    }
}

pub struct StartupState;

impl State<AppTransition> for StartupState {
    fn run(self: Box<Self>) -> AppTransition {
        let events_loop = glutin::EventsLoop::new();

        let window = glutin::WindowBuilder::new()
            .with_dimensions(glutin::dpi::LogicalSize::new(800.0, 600.0))
            .with_title("crufty".to_string());

        let context = glutin::ContextBuilder::new();

        let display = glium::Display::new(window, context, &events_loop).unwrap();

        AppTransition::StartGame(display, events_loop)
    }
}

pub struct ShutdownState;

impl State<AppTransition> for ShutdownState {
    fn run(self: Box<Self>) -> AppTransition {
        AppTransition::TerminateApplication
    }
}
