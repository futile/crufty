use std::collections::HashMap;
use std::intrinsics::type_id;
use std::net::Ipv4Addr;

use bincode::{deserialize_from};

use ecs::{World};

use enet::{self, Event};

use crate::components::{Position};
use crate::systems::LevelSystems;
use super::{PORT, UPDATE_CHANNEL_ID, ENET};

pub struct Client {
    enet_host: enet::Host<()>,
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

        Client { enet_host }
    }

    fn deserialize_updates(data: &[u8], world: &mut World<LevelSystems>) {
        let mut reader = std::io::Cursor::new(data);

        while reader.get_ref().len() - (reader.position() as usize) > 0 {
            let tag: u64 = deserialize_from(&mut reader).unwrap();

            loop {
                let e_id: u64 = deserialize_from(&mut reader).unwrap();
                let sim_ts: u64 = deserialize_from(&mut reader).unwrap();

                let pos_type_id = unsafe { type_id::<Position>() };
                match tag {
                    id if id == pos_type_id => {
                        let pos: Position = deserialize_from(&mut reader).unwrap();
                        println!("received 'Position' update for entity {} (at {}): {:#?}", e_id, sim_ts, pos);
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
        let maybe_event = self.enet_host.service(0).unwrap() ;
        if let Some(event) = maybe_event {
            if let Event::Receive { channel_id: UPDATE_CHANNEL_ID, ref packet, .. } = event {
                let data = packet.data();
                println!("received {} bytes updates", data.len());
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
