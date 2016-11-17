use std::time::Duration;

use util::{State, Transition};
use net::udp::{UdpConnection, ReceiveEvent};

mod run_state;

use self::run_state::ClientRunState;

pub enum ClientTransition {
    Startup,
    StartGame(UdpConnection),
    Shutdown,
    TerminateClient,
}

impl Transition for ClientTransition {
    fn create_state(self) -> Option<Box<State<ClientTransition>>> {
        match self {
            ClientTransition::Startup => Some(Box::new(ClientStartupState)),
            ClientTransition::StartGame(conn) => Some(Box::new(ClientRunState::new(conn))),
            ClientTransition::Shutdown => Some(Box::new(ClientShutdownState)),
            ClientTransition::TerminateClient => None,
        }
    }
}

pub struct ClientStartupState;
pub struct ClientShutdownState;

impl State<ClientTransition> for ClientStartupState {
    fn run(self: Box<Self>) -> ClientTransition {
        let mut conn = UdpConnection::new(&"127.0.0.1:12365".parse().unwrap(),
                                      &"127.0.0.1:12366".parse().unwrap(),
                                      Duration::from_secs(1));

        println!("client: connecting to server..");
        conn.send_bytes(&[]).unwrap();

        ClientTransition::StartGame(conn)
    }
}

impl State<ClientTransition> for ClientShutdownState {
    fn run(self: Box<Self>) -> ClientTransition {
        ClientTransition::TerminateClient
    }
}
