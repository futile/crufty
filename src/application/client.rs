use glium::{self, glutin};

use crate::util::{State, Transition};
use super::GameState;

pub enum ClientTransition {
    Startup,
    StartGame(glium::Display, glutin::EventsLoop),
    Shutdown,
    TerminateApplication,
}

impl Transition for ClientTransition {
    fn create_state(self) -> Option<Box<dyn State<ClientTransition>>> {
        match self {
            ClientTransition::Startup => Some(Box::new(StartupState)),
            ClientTransition::StartGame(d, el) => Some(Box::new(GameState::new(d, el))),
            ClientTransition::Shutdown => Some(Box::new(ShutdownState)),
            ClientTransition::TerminateApplication => None,
        }
    }
}

pub struct StartupState;

impl State<ClientTransition> for StartupState {
    fn run(self: Box<Self>) -> ClientTransition {
        let events_loop = glutin::EventsLoop::new();

        let window = glutin::WindowBuilder::new()
            .with_dimensions(glutin::dpi::LogicalSize::new(800.0, 600.0))
            .with_title("crufty".to_string());

        let context = glutin::ContextBuilder::new().with_depth_buffer(24);

        let display = glium::Display::new(window, context, &events_loop).unwrap();

        ClientTransition::StartGame(display, events_loop)
    }
}

pub struct ShutdownState;

impl State<ClientTransition> for ShutdownState {
    fn run(self: Box<Self>) -> ClientTransition {
        ClientTransition::TerminateApplication
    }
}
