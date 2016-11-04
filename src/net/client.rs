use std::str::FromStr;
use std::io;
use std::net::SocketAddr;

use mio::udp::UdpSocket;

use super::{ClientId, Message};

#[derive(Debug)]
pub enum ConnectionState {
    Disconnected,
    Connecting(SocketAddr),
    Connected(SocketAddr, ClientId),
}

#[derive(Debug)]
pub struct ClientConnection {
    socket: UdpSocket,
    state: ConnectionState,
    buffer: Vec<u8>,
}

#[derive(Debug)]
pub enum ConnectionEvent {
    ConnectedToServer(ClientId),
}

impl ClientConnection {
    pub fn new() -> io::Result<ClientConnection> {
        let socket = UdpSocket::bind(&SocketAddr::from_str("127.0.0.1:0").unwrap())?;

        Ok(ClientConnection {
            socket: socket,
            state: ConnectionState::Disconnected,
            buffer: vec![0; 20],
        })
    }

    fn send_to_addr(&mut self, server: &SocketAddr, message: Message) {
        let msg = message.encode();

        let mut remaining = msg.len();
        while remaining > 0 {
            if let Some(sent) = self.socket.send_to(&msg, server).unwrap() {
                remaining -= sent;
            }
        }

        println!("sent message: {:?}", message);
    }

    pub fn start_connect(&mut self, server_addr: &SocketAddr) {
        self.state = ConnectionState::Connecting(server_addr.clone());

        self.send_to_addr(server_addr, Message::Connect);
    }

    pub fn handle(&mut self) -> Option<ConnectionEvent> {
        let (read_size, addr) = match self.socket.recv_from(&mut self.buffer).unwrap() {
            None => return None,
            Some(some) => some,
        };

        let msg = Message::decode(&self.buffer[..read_size]);

        println!("received message: {:?}", msg);

        let mut maybe_new_state: Option<ConnectionState> = None;

        if let ConnectionState::Connecting(connecting_addr) = self.state {
            if connecting_addr != addr {
                println!("warning: expected reply from server('{}'), but got a reply from '{}', \
                          ignoring.",
                         connecting_addr,
                         addr);
                return None;
            }

            if let Message::ConnectResponse(client_id) = msg {
                maybe_new_state = Some(ConnectionState::Connected(addr, client_id));

                self.send_to_addr(&connecting_addr, Message::Ack);
            }
        }

        if let Some(new_state) = maybe_new_state {
            self.state = new_state;

            if let ConnectionState::Connected(_, id) = self.state {
                return Some(ConnectionEvent::ConnectedToServer(id));
            }
        }

        None
    }
}
