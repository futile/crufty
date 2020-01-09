use std::intrinsics::type_id;
use std::net::Ipv4Addr;
use std::collections::HashMap;

use bincode::deserialize_from;

use ecs::{World, Entity, ModifyData};

use enet::{self, Event};

use super::{ENET, PORT, UPDATE_CHANNEL_ID};
use crate::components::*;
use crate::systems::LevelSystems;

pub struct Client {
    enet_host: enet::Host<()>,
    entity_mapping: HashMap<u64, Entity>
}

const POSITION_ID: u64 = type_id::<Position>();
const VELOCITY_ID: u64 = type_id::<Velocity>();
const JUMP_ID: u64 = type_id::<Jump>();
const GRAVITY_ID: u64 = type_id::<Gravity>();
const FACING_ID: u64 = type_id::<Facing>();
const INTENTS_ID: u64 = type_id::<Intents>();
const INTERACTOR_ID: u64 = type_id::<Interactor>();
const CAMERA_ID: u64 = type_id::<Camera>();
const MOVEMENT_ID: u64 = type_id::<Movement>();
const SPRITE_ID: u64 = type_id::<Sprite>();

macro_rules! deserialize_component {
    ($component:ident, $component_name:ident, $en:ident, $e_id:ident, $sim_ts:ident, $world:ident, $reader:ident, $do_print:expr) => (
        {
            let v: $component = deserialize_from(&mut $reader).unwrap();
            if $do_print {
                println!(
                    "received '{}' update for entity {} (at {}): {:#?}",
                    stringify!($component), $e_id, $sim_ts, v
                );
            }
            $world.modify_entity($en, move |e: ModifyData<LevelComponents>, data: &mut LevelComponents| {
                data.$component_name.insert(&e, v);
            });
        }
    );
}

impl Client {
    pub fn new() -> Client {
        let enet_host = ENET
            .create_host::<()>(
                None,
                1,
                enet::ChannelLimit::Maximum,
                enet::BandwidthLimit::Unlimited,
                enet::BandwidthLimit::Unlimited,
            )
            .expect("could not create host");

        Client {
            enet_host,
            entity_mapping: HashMap::new()
        }
    }

    fn deserialize_updates(&mut self, data: &[u8], world: &mut World<LevelSystems>) {
        let mut reader = std::io::Cursor::new(data);

        while reader.get_ref().len() - (reader.position() as usize) > 0 {
            let tag: u64 = deserialize_from(&mut reader).unwrap();

            loop {
                let e_id: u64 = deserialize_from(&mut reader).unwrap();
                let sim_ts: u64 = deserialize_from(&mut reader).unwrap();

                let en = *self.entity_mapping.entry(e_id).or_insert_with(|| {
                    let e = world.create_entity(());
                    println!("e_id: {}, e: {:?}", e_id, e);
                    e
                });

                match tag {
                    POSITION_ID => deserialize_component!(Position, position, en, e_id, sim_ts, world, reader, false),
                    VELOCITY_ID => deserialize_component!(Velocity, velocity, en, e_id, sim_ts, world, reader, false),
                    JUMP_ID => deserialize_component!(Jump, jump, en, e_id, sim_ts, world, reader, true),
                    GRAVITY_ID => deserialize_component!(Gravity, gravity, en, e_id, sim_ts, world, reader, true),
                    FACING_ID => deserialize_component!(Facing, facing, en, e_id, sim_ts, world, reader, true),
                    INTENTS_ID => deserialize_component!(Intents, intents, en, e_id, sim_ts, world, reader, false),
                    INTERACTOR_ID => deserialize_component!(Interactor, interactor, en, e_id, sim_ts, world, reader, true),
                    CAMERA_ID => deserialize_component!(Camera, camera, en, e_id, sim_ts, world, reader, true),
                    MOVEMENT_ID => deserialize_component!(Movement, movement, en, e_id, sim_ts, world, reader, false),
                    SPRITE_ID => deserialize_component!(Sprite, sprite, en, e_id, sim_ts, world, reader, false),
                    _ => panic!("unexpected type_id: {}", tag),
                }

                let more: bool = deserialize_from(&mut reader).unwrap();
                if !more {
                    break;
                }
            }
        }
    }

    pub fn maintain(&mut self, world: &mut World<LevelSystems>) {
        let data = {
            let maybe_event = self.enet_host.service(0).unwrap();
            if let Some(event) = maybe_event {
                if let Event::Receive {
                    channel_id: UPDATE_CHANNEL_ID,
                    ref packet,
                    ..
                } = event
                {
                    let data = packet.data().to_vec();
                    assert!(!data.is_empty());
                    Some(data)
                } else {
                    dbg!(event);
                    None
                }
            } else {
                None
            }
        };

        if let Some(data) = data {
            self.deserialize_updates(&data, world);
        }
    }

    pub fn start_connect(&mut self, dest_addr: Ipv4Addr) {
        self.enet_host
            .connect(&enet::Address::new(dest_addr, PORT), 10, 0)
            .unwrap();
    }
}
