use glium::{self, glutin};

use crate::util::{State, Transition};

mod gamestate;
mod input;

pub use self::gamestate::GameState;
pub use self::input::{InputContext, InputIntent, InputManager, InputState, KeyHandler};

pub mod server {
    use super::*;

    pub enum ServerTransition {
        Startup,
        StartGame(glium::Display, glutin::EventsLoop),
        Shutdown,
        TerminateApplication,
    }

    impl Transition for ServerTransition {
        fn create_state(self) -> Option<Box<dyn State<ServerTransition>>> {
            match self {
                ServerTransition::Startup => Some(Box::new(StartupState)),
                ServerTransition::StartGame(d, el) => Some(Box::new(GameState::new(d, el))),
                ServerTransition::Shutdown => Some(Box::new(ShutdownState)),
                ServerTransition::TerminateApplication => None,
            }
        }
    }

    pub struct StartupState;

    impl State<ServerTransition> for StartupState {
        fn run(self: Box<Self>) -> ServerTransition {
            let events_loop = glutin::EventsLoop::new();

            let window = glutin::WindowBuilder::new()
                .with_dimensions(glutin::dpi::LogicalSize::new(800.0, 600.0))
                .with_title("crufty".to_string());

            let context = glutin::ContextBuilder::new().with_depth_buffer(24);

            let display = glium::Display::new(window, context, &events_loop).unwrap();

            ServerTransition::StartGame(display, events_loop)
        }
    }

    pub struct ShutdownState;

    impl State<ServerTransition> for ShutdownState {
        fn run(self: Box<Self>) -> ServerTransition {
            ServerTransition::TerminateApplication
        }
    }
}

pub mod client {
    use super::*;

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
}
