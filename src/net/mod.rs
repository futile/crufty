pub mod server;
pub mod client;

pub type ClientId = u16;

use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

#[derive(Debug, PartialEq, Eq)]
pub enum Message {
    Ack,
    Connect,
    ConnectResponse(ClientId),
}

impl Message {
    pub fn encode(&self) -> Vec<u8> {
        let mut msg: Vec<u8> = vec![];

        let id: u16 = match *self {
            Message::Ack => 1,
            Message::Connect => 11,
            Message::ConnectResponse(_) => 12,
        };

        msg.write_u16::<LittleEndian>(id).unwrap();

        if let Message::ConnectResponse(client_id) = *self {
            msg.write_u16::<LittleEndian>(client_id).unwrap();
        }

        return msg;
    }

    pub fn decode(msg: &[u8]) -> Message {
        let mut reader = Cursor::new(msg);

        let msg_type = reader.read_u16::<LittleEndian>().unwrap();

        match msg_type {
            1 => Message::Ack,
            11 => Message::Connect,
            12 => {
                let client_id = reader.read_u16::<LittleEndian>().unwrap();
                Message::ConnectResponse(client_id)
            }
            _ => panic!("unexpected msg_type: {}", msg_type),
        }
    }
}
