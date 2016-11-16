use util::{State, Transition};

pub enum ServerTransition {
    Startup,
    StartGame,
    Shutdown,
    TerminateServer,
}

impl Transition for ServerTransition {
    fn create_state(self) -> Option<Box<State<ServerTransition>>> {
        match self {
            ServerTransition::Startup => Some(Box::new(ServerStartupState)),
            ServerTransition::StartGame => Some(Box::new(ServerRunState)),
            ServerTransition::Shutdown => Some(Box::new(ServerShutdownState)),
            ServerTransition::TerminateServer => None,
        }
    }
}

pub struct ServerStartupState;
pub struct ServerRunState;
pub struct ServerShutdownState;

impl State<ServerTransition> for ServerStartupState {
    fn run(self: Box<Self>) -> ServerTransition {
        ServerTransition::StartGame
    }
}

impl State<ServerTransition> for ServerRunState {
    fn run(self: Box<Self>) -> ServerTransition {
        ServerTransition::Shutdown
    }
}

impl State<ServerTransition> for ServerShutdownState {
    fn run(self: Box<Self>) -> ServerTransition {
        ServerTransition::TerminateServer
    }
}
