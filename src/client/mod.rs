use std::time::Duration;

use util::{State, Transition};
use net::udp::{ReceiveEvent};

mod run_state;
pub mod net;

use self::run_state::ClientRunState;
use self::net::NetworkInterface;

pub enum ClientTransition {
    Startup,
    StartGame(NetworkInterface),
    Shutdown,
    TerminateClient,
}

impl Transition for ClientTransition {
    fn create_state(self) -> Option<Box<State<ClientTransition>>> {
        match self {
            ClientTransition::Startup => Some(Box::new(ClientStartupState)),
            ClientTransition::StartGame(niface) => Some(Box::new(ClientRunState::new(niface))),
            ClientTransition::Shutdown => Some(Box::new(ClientShutdownState)),
            ClientTransition::TerminateClient => None,
        }
    }
}

pub struct ClientStartupState;
pub struct ClientShutdownState;

impl State<ClientTransition> for ClientStartupState {
    fn run(self: Box<Self>) -> ClientTransition {
        let mut niface = NetworkInterface::new(&"127.0.0.1:12365".parse().unwrap(),
                                           &"127.0.0.1:12366".parse().unwrap()).unwrap();

        println!("client: connecting to server (sending empty packet)..");
        niface.wrapper.wrap_and_send_payload(&[], &mut niface.conn).unwrap();

        ClientTransition::StartGame(niface)
    }
}

impl State<ClientTransition> for ClientShutdownState {
    fn run(self: Box<Self>) -> ClientTransition {
        ClientTransition::TerminateClient
    }
}
