use std::io::Cursor;
use std::io::prelude::{Write, Read};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use specs::{self, Join, InsertResult};

use mincode::{SizeLimit, FloatEncoding};
use mincode::rustc_serialize::{encode_into, decode_from};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

#[derive(Copy,Clone,Debug, RustcEncodable, RustcDecodable)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl specs::Component for Position {
    type Storage = specs::VecStorage<Position>;
}

#[derive(Clone,Copy,Debug)]
pub struct Info(pub i32);

#[derive(Clone, Debug, Default)]
pub struct CContext {
    pub positions: Arc<Mutex<HashMap<specs::Entity, Position>>>,
}

pub fn serialize_ccontext(context: &CContext) -> Vec<u8> {
    // write header for Position
    // serialize all Positions
    let mut writer = Vec::new();

    let positions = context.positions.lock().unwrap();
    let count: usize = positions.len();

    if count > ::std::u16::MAX as usize {
        panic!("serialize_world(): count > u16::MAX");
    }

    writer.write_u16::<BigEndian>(count as u16).unwrap();

    for (e, p) in positions.iter() {
        encode_into(&e, &mut writer, SizeLimit::Infinite, FloatEncoding::Normal).unwrap();
        encode_into(&p, &mut writer, SizeLimit::Infinite, FloatEncoding::Normal).unwrap();
    }

    writer
}

#[derive(Debug, Default)]
pub struct WorldSyncer {
    entity_mapping: HashMap<specs::Entity, specs::Entity>,
}

impl WorldSyncer {
    pub fn deserialize_into_world(&mut self, world: &mut specs::World<()>, ser: &[u8]) {
        let mut reader = Cursor::new(ser);

        let pos_count = reader.read_u16::<BigEndian>().unwrap();

        println!("pos_count: {}", pos_count);

        for _ in 0..pos_count {
            let server_ent: specs::Entity =
                decode_from(&mut reader, SizeLimit::Infinite, FloatEncoding::Normal).unwrap();
            let pos: Position =
                decode_from(&mut reader, SizeLimit::Infinite, FloatEncoding::Normal).unwrap();

            let local_ent = self.entity_mapping
                .entry(server_ent)
                .or_insert_with(|| world.create_now().build());

            if let InsertResult::EntityIsDead(_) = world.write::<Position>()
                .insert(*local_ent, pos) {
                panic!("deserialize_into_world(): entity is dead locally");
            }
        }
    }
}
