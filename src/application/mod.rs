use glium;
use glutin;

use util::{State, Transition};

pub enum AppTransition {
    Startup,
    StartGame(glium::Display),
    Shutdown,
    TerminateApplication
}

impl Transition for AppTransition {
    fn initial() -> AppTransition {
        AppTransition::Startup
    }

    fn create_state(self) -> Option<Box<State<AppTransition>>> {
        match self {
            AppTransition::Startup => Some(Box::new(StartupState)),
            AppTransition::StartGame(d) => Some(Box::new(GameState{display: d})),
            AppTransition::Shutdown => Some(Box::new(ShutdownState)),
            AppTransition::TerminateApplication => None
        }
    }
}

pub struct StartupState;

impl State<AppTransition> for StartupState {
    fn run(self: Box<Self>) -> AppTransition {
        use glium::DisplayBuild;

        let display = glutin::WindowBuilder::new()
            .with_dimensions(800, 600)
            .with_title("Crufty".to_string())
            .build_glium().unwrap();

        AppTransition::StartGame(display)
    }
}

pub struct GameState {
    display: glium::Display,
}

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
