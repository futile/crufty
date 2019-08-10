use std::io::Cursor;

use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessageType {
    EntityUpdates,
}

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageHeader {
    message_type: MessageType,
}

pub trait MessageVisitor {
    fn visit_entity_updates(&mut self, data: &mut Cursor<&[u8]>);
}

pub fn parse_and_visit_message<V: MessageVisitor>(message: &[u8], mut visitor: V) {
    let mut reader = Cursor::new(message);
    let header: MessageHeader = bincode::deserialize_from(&mut reader).unwrap();

    match header.message_type {
        MessageType::EntityUpdates => visitor.visit_entity_updates(&mut reader),
    }
}
