use std::intrinsics::type_id;
use std::net::Ipv4Addr;

use bincode::deserialize_from;

use ecs::World;

use enet::{self, Event};

use super::{ENET, PORT, UPDATE_CHANNEL_ID};
use crate::components::*;
use crate::systems::LevelSystems;

pub struct Client {
    enet_host: enet::Host<()>,
}

const POSITION_ID: u64 = unsafe { type_id::<Position>() };
const VELOCITY_ID: u64 = unsafe { type_id::<Velocity>() };
const JUMP_ID: u64 = unsafe { type_id::<Jump>() };
const GRAVITY_ID: u64 = unsafe { type_id::<Gravity>() };
const FACING_ID: u64 = unsafe { type_id::<Facing>() };
const INTENTS_ID: u64 = unsafe { type_id::<Intents>() };
const INTERACTOR_ID: u64 = unsafe { type_id::<Interactor>() };

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

        Client { enet_host }
    }

    fn deserialize_updates(data: &[u8], world: &mut World<LevelSystems>) {
        let mut reader = std::io::Cursor::new(data);

        while reader.get_ref().len() - (reader.position() as usize) > 0 {
            let tag: u64 = deserialize_from(&mut reader).unwrap();

            loop {
                let e_id: u64 = deserialize_from(&mut reader).unwrap();
                let sim_ts: u64 = deserialize_from(&mut reader).unwrap();

                match tag {
                    POSITION_ID => {
                        let pos: Position = deserialize_from(&mut reader).unwrap();
                        println!(
                            "received 'Position' update for entity {} (at {}): {:#?}",
                            e_id, sim_ts, pos
                        );
                    }
                    VELOCITY_ID => {
                        let vel: Velocity = deserialize_from(&mut reader).unwrap();
                        println!(
                            "received 'Velocity' update for entity {} (at {}): {:#?}",
                            e_id, sim_ts, vel
                        );
                    }
                    JUMP_ID => {
                        let jump: Jump = deserialize_from(&mut reader).unwrap();
                        println!(
                            "received 'Jump' update for entity {} (at {}): {:#?}",
                            e_id, sim_ts, jump
                        );
                    }
                    GRAVITY_ID => {
                        let grav: Gravity = deserialize_from(&mut reader).unwrap();
                        println!(
                            "received 'Gravity' update for entity {} (at {}): {:#?}",
                            e_id, sim_ts, grav
                        );
                    }
                    FACING_ID => {
                        let facing: Facing = deserialize_from(&mut reader).unwrap();
                        println!(
                            "received 'Facing' update for entity {} (at {}): {:#?}",
                            e_id, sim_ts, facing
                        );
                    }
                    INTENTS_ID => {
                        let intents: Intents = deserialize_from(&mut reader).unwrap();
                        println!(
                            "received 'Intents' update for entity {} (at {}): {:#?}",
                            e_id, sim_ts, intents
                        );
                    }
                    INTERACTOR_ID => {
                        let interactor: Interactor = deserialize_from(&mut reader).unwrap();
                        println!(
                            "received 'Interactor' update for entity {} (at {}): {:#?}",
                            e_id, sim_ts, interactor
                        );
                    }
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
        let maybe_event = self.enet_host.service(0).unwrap();
        if let Some(event) = maybe_event {
            if let Event::Receive {
                channel_id: UPDATE_CHANNEL_ID,
                ref packet,
                ..
            } = event
            {
                let data = packet.data();
                assert!(data.len() > 0);
                Client::deserialize_updates(data, world);
            } else {
                dbg!(event);
            }
        }
    }

    pub fn start_connect(&mut self, dest_addr: Ipv4Addr) {
        self.enet_host
            .connect(&enet::Address::new(dest_addr, PORT), 10, 0)
            .unwrap();
    }
}
