use util::State;
use net::udp::{UdpConnection};

use super::ClientTransition;

pub struct ClientRunState {
    conn: UdpConnection,
}

impl ClientRunState {
    pub fn new(conn: UdpConnection) -> ClientRunState {
        ClientRunState {
            conn: conn,
        }
    }
}

impl State<ClientTransition> for ClientRunState {
    fn run(self: Box<Self>) -> ClientTransition {
        unimplemented!()
    }
}
