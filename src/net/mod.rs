use std::net::Ipv4Addr;
use std::time::{Instant, Duration};
use std::collections::HashMap;

use ecs::{World, Entity};

use enet::{self, Enet, Event};

use crate::components::{Position};
use crate::systems::LevelSystems;

lazy_static! {
    static ref ENET: Enet = Enet::new().unwrap();
}

const PORT: u16 = 9001;
const RESEND_DURATION: Duration = Duration::from_millis(100);

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
                res.positions.insert(**en, (pos, world.services.simulation_time, Instant::now()));
            }
        };

        return res;
    }

    fn serialize_updates(&mut self) -> Vec<u8> {
        let now = Instant::now();
        // let mut data = vec![];

        for (e, pos_update) in &mut self.positions {
            if pos_update.2 > now {
                continue;
            }

            pos_update.2 = now + RESEND_DURATION;
        };

        unimplemented!()
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
                Event::Connect(ref mut peer) => peer.set_data(Some(PeerData::new_from_world(world))),
                _ => (),
            }
        };

        if let Some(event) = self.enet_host.service(0).unwrap() {
            self.last_maintain = Instant::now();

            loop_body(event, world);
        };

        while let Some(event) = self.enet_host.check_events().unwrap() {
            loop_body(event, world);
        };
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
