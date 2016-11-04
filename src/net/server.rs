use std::io;
use std::net::SocketAddr;
use std::collections::HashMap;

use mio::udp::UdpSocket;

use super::{Message, ClientId};

#[derive(Debug, Clone)]
pub struct ClientState {
    id: ClientId,
}

impl ClientState {
    pub fn new(id: ClientId) -> ClientState {
        ClientState { id: id }
    }
}

#[derive(Debug, Clone)]
pub enum ConnectionState {
    Connecting(ClientState),
    Connected(ClientState),
}

#[derive(Debug)]
pub struct ServerBind {
    socket: UdpSocket,
    clients: HashMap<SocketAddr, ConnectionState>,
    next_id: ClientId,
    buffer: Vec<u8>,
}

#[derive(Debug)]
pub enum NetworkEvent {
    ClientConnected(ClientId),
}

impl ServerBind {
    pub fn new(addr: &SocketAddr) -> io::Result<ServerBind> {
        let socket = UdpSocket::bind(addr)?;

        Ok(ServerBind {
            socket: socket,
            clients: HashMap::new(),
            next_id: 1,
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

    pub fn handle(&mut self) -> Option<NetworkEvent> {
        let (read_size, client_addr) = match self.socket.recv_from(&mut self.buffer).unwrap() {
            None => return None,
            Some(some) => some,
        };

        let msg = Message::decode(&self.buffer[..read_size]);

        println!("received message: {:?}", msg);

        if !self.clients.contains_key(&client_addr) {
            if let Message::Connect = msg {
                let client_id = self.next_id;
                self.next_id += 1;

                self.clients.insert(client_addr,
                                    ConnectionState::Connecting(ClientState::new(client_id)));

                self.send_to_addr(&client_addr, Message::ConnectResponse(client_id));
            } else {
                println!("warning: unconnected client, but message '{:?}' was received, ignoring.",
                         msg);
            }

            return None;
        }

        let state = self.clients.get_mut(&client_addr).unwrap().clone();

        match state {
            ConnectionState::Connecting(ref cs) if msg == Message::Ack => {
                self.clients.insert(client_addr, ConnectionState::Connected(cs.clone()));

                self.send_to_addr(&client_addr, Message::Ack);

                return Some(NetworkEvent::ClientConnected(cs.id));
            }
            _ => {
                println!("warning: unexpected message '{:?}' from client in state '{:?}', \
                          ignoring.",
                         msg,
                         state)
            }
        };

        None
    }
}
