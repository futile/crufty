use std::time::Duration;

use util::{State, Transition};
use net::udp::{ReliabilityWrapper, ReceiveEvent};

mod run_state;
mod net;

use self::run_state::ServerRunState;
use self::net::NetworkInterface;

pub enum ServerTransition {
    Startup,
    StartGame(NetworkInterface),
    Shutdown,
    TerminateServer,
}

impl Transition for ServerTransition {
    fn create_state(self) -> Option<Box<State<ServerTransition>>> {
        match self {
            ServerTransition::Startup => Some(Box::new(ServerStartupState)),
            ServerTransition::StartGame(niface) => Some(Box::new(ServerRunState::new(niface))),
            ServerTransition::Shutdown => Some(Box::new(ServerShutdownState)),
            ServerTransition::TerminateServer => None,
        }
    }
}

pub struct ServerStartupState;
pub struct ServerShutdownState;

impl State<ServerTransition> for ServerStartupState {
    fn run(self: Box<Self>) -> ServerTransition {
        let mut niface = NetworkInterface::new(&"127.0.0.1:12366".parse().unwrap());

        println!("server: waiting for connection..");

        // wait for a connection before really starting
        niface.perform_receive_phase(None).unwrap();

        println!("got a connection, starting main loop");

        ServerTransition::StartGame(niface)
    }
}

impl State<ServerTransition> for ServerShutdownState {
    fn run(self: Box<Self>) -> ServerTransition {
        ServerTransition::TerminateServer
    }
}
