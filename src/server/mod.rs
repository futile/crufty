use std::time::Duration;

use util::{State, Transition};
use net::udp::{UdpConnection, ReceiveEvent};

mod run_state;

use self::run_state::ServerRunState;

pub enum ServerTransition {
    Startup,
    StartGame(UdpConnection),
    Shutdown,
    TerminateServer,
}

impl Transition for ServerTransition {
    fn create_state(self) -> Option<Box<State<ServerTransition>>> {
        match self {
            ServerTransition::Startup => Some(Box::new(ServerStartupState)),
            ServerTransition::StartGame(conn) => Some(Box::new(ServerRunState::new(conn))),
            ServerTransition::Shutdown => Some(Box::new(ServerShutdownState)),
            ServerTransition::TerminateServer => None,
        }
    }
}

pub struct ServerStartupState;
pub struct ServerShutdownState;

impl State<ServerTransition> for ServerStartupState {
    fn run(self: Box<Self>) -> ServerTransition {
        let mut conn = UdpConnection::new(&"127.0.0.1:12366".parse().unwrap(),
                                      &"127.0.0.1:12365".parse().unwrap(),
                                      Duration::from_secs(1));

        println!("server: waiting for connection..");

        // wait for a connection before really starting
        conn.recv_with_timeout(None, |e| match e {
            ReceiveEvent::NewAck(msg_id, _) => println!("u what m8? {:?}", msg_id),
            ReceiveEvent::NewData(data) => assert_eq!(data, &[]),
        });

        println!("got a connection, starting main loop");

        ServerTransition::StartGame(conn)
    }
}

impl State<ServerTransition> for ServerShutdownState {
    fn run(self: Box<Self>) -> ServerTransition {
        ServerTransition::TerminateServer
    }
}
