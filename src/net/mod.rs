use std::net::Ipv4Addr;
use std::time::Instant;

use ecs::World;

use enet::{self, Enet, Event};

use crate::components::LevelChangedFlags;
use crate::systems::LevelSystems;

lazy_static! {
    static ref ENET: Enet = Enet::new().unwrap();
}

const PORT: u16 = 9001;

#[derive(Debug, Default)]
struct PeerData {
    changes: LevelChangedFlags,
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
        fn loop_body(mut event: Event<'_, PeerData>) {
            dbg!(&event);

            match event {
                Event::Connect(ref mut peer) => peer.set_data(Some(PeerData::default())),
                _ => (),
            }
        };

        if let Some(mut event) = self.enet_host.service(0).unwrap() {
            loop_body(event);
        };

        while let Some(mut event) = self.enet_host.check_events().unwrap() {
            loop_body(event);
        };

        self.last_maintain = Instant::now();
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
