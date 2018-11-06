use std::collections::HashMap;
use std::intrinsics::type_id;
use std::net::Ipv4Addr;
use std::time::{Duration, Instant};

use bincode::serialize_into;
use ecs::{Entity, World};

use enet::{self, Enet, Event, Packet, PacketMode, PeerState};

use crate::components::Position;
use crate::systems::LevelSystems;

lazy_static! {
    static ref ENET: Enet = Enet::new().unwrap();
}

const PORT: u16 = 9001;
const RESEND_DURATION: Duration = Duration::from_secs(10);
const UPDATE_CHANNEL_ID: u8 = 1;

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

    fn serialize_updates(&mut self) -> Option<Vec<u8>> {
        let now = Instant::now();
        let mut data = vec![];

        let mut tag_written = false;

        for (e, pos_update) in &mut self.positions {
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

pub struct Host {
    enet_host: enet::Host<PeerData>,
    last_maintain: Instant,
}

impl Host {
    pub fn new() -> Host {
        let enet_host = ENET
            .create_host(
                Some(&enet::Address::new(Ipv4Addr::LOCALHOST, PORT)),
                255,
                enet::ChannelLimit::Maximum,
                enet::BandwidthLimit::Unlimited,
                enet::BandwidthLimit::Unlimited,
            )
            .unwrap();

        Host {
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

            if let Some(update_data) = peer.data_mut().unwrap().serialize_updates() {
                peer.send_packet(
                    Packet::new(&update_data, PacketMode::UnreliableUnsequenced).unwrap(),
                    UPDATE_CHANNEL_ID,
                )
                .unwrap();
            }
        }
    }
}

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

    pub fn maintain(&mut self, world: &mut World<LevelSystems>) {
        if let Some(event) = self.enet_host.service(0).unwrap() {
            dbg!(event);
        }
    }

    pub fn start_connect(&mut self, dest_addr: Ipv4Addr) {
        self.enet_host
            .connect(&enet::Address::new(dest_addr, PORT), 10, 0)
            .unwrap();
    }
}
