use std::collections::HashMap;
use std::intrinsics::type_id;
use std::net::Ipv4Addr;
use std::time::{Instant};

use bincode::{serialize_into};
use ecs::{Entity, World};

use enet::{self, Event, Packet, PacketMode, PeerState};

use crate::components::{Position};
use crate::systems::LevelSystems;
use super::{PORT, RESEND_DURATION, UPDATE_CHANNEL_ID, ENET};

type UpdateMap<C> = HashMap<Entity, (C, u64, Instant)>;

#[derive(Debug, Default)]
struct PeerData {
    positions: UpdateMap<Position>,
}

impl PeerData {
    fn new_from_world(world: &mut World<LevelSystems>) -> PeerData {
        let mut res = PeerData::default();

        for en in world.entities() {
            if let Some(pos) = world.position.get(&en) {
                res.positions
                    .insert(**en, (pos, world.services.simulation_time, Instant::now()));
            }
        }

        return res;
    }

    fn update_from_changes(&mut self, world: &mut World<LevelSystems>) {
        let sim_time = world.services.simulation_time;
        for (e, pos) in world.services.changed_flags.position.drain() {
            self.positions.insert(e, (pos, sim_time, Instant::now()));
        }
    }

    fn serialize_updates(&mut self) -> Option<Vec<u8>> {
        let now = Instant::now();
        let mut data = vec![];

        let mut tag_written = false;

        // TODO remove drain() and instead send responses from client
        for (e, mut pos_update) in self.positions.drain() {
            if pos_update.2 > now {
                continue;
            }

            pos_update.2 = now + RESEND_DURATION;

            if !tag_written {
                tag_written = true;
                serialize_into(&mut data, unsafe { &type_id::<Position>() }).unwrap();
            } else {
                serialize_into(&mut data, &true).unwrap();
            }

            serialize_into(&mut data, &e.id()).unwrap();
            serialize_into(&mut data, &pos_update.1).unwrap();
            serialize_into(&mut data, &pos_update.0).unwrap();
        }

        if tag_written {
            serialize_into(&mut data, &false).unwrap();
        }

        if data.is_empty() {
            None
        } else {
            Some(data)
        }
    }
}

pub struct Server {
    enet_host: enet::Host<PeerData>,
    last_maintain: Instant,
}

impl Server {
    pub fn new() -> Server {
        let enet_host = ENET
            .create_host(
                Some(&enet::Address::new(Ipv4Addr::LOCALHOST, PORT)),
                255,
                enet::ChannelLimit::Maximum,
                enet::BandwidthLimit::Unlimited,
                enet::BandwidthLimit::Unlimited,
            )
            .unwrap();

        Server {
            enet_host,
            last_maintain: Instant::now(),
        }
    }

    pub fn maintain(&mut self, world: &mut World<LevelSystems>) {
        fn loop_body(mut event: Event<'_, PeerData>, world: &mut World<LevelSystems>) {
            dbg!(&event);

            match event {
                Event::Connect(ref mut peer) => {
                    peer.set_data(Some(PeerData::new_from_world(world)))
                }
                _ => (),
            }
        };

        if let Some(event) = self.enet_host.service(0).unwrap() {
            self.last_maintain = Instant::now();

            loop_body(event, world);
        };

        while let Some(event) = self.enet_host.check_events().unwrap() {
            loop_body(event, world);
        }

        for mut peer in self.enet_host.peers() {
            if peer.state() != PeerState::Connected {
                continue;
            }

            let data = peer.data_mut().unwrap();
            data.update_from_changes(world);

            if let Some(update_data) = data.serialize_updates() {
                println!("sending {} bytes", update_data.len());
                peer.send_packet(
                    Packet::new(&update_data, PacketMode::UnreliableUnsequenced).unwrap(),
                    UPDATE_CHANNEL_ID,
                )
                .unwrap();
            }
        }
    }
}
