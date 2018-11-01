use std::net::Ipv4Addr;

use enet::{self, Enet};

lazy_static! {
    static ref ENET: Enet = Enet::new().unwrap();
}

const PORT: u16 = 9001;

pub struct Host {
    enet_host: enet::Host<()>,
}

impl Host {
    pub fn new() -> Host {
        let enet_host = ENET
            .create_host::<()>(
                Some(&enet::Address::new(Ipv4Addr::LOCALHOST, PORT)),
                255,
                enet::ChannelLimit::Maximum,
                enet::BandwidthLimit::Unlimited,
                enet::BandwidthLimit::Unlimited,
            )
            .unwrap();

        Host { enet_host }
    }

    pub fn maintain(&mut self) {
        if let Some(event) = self.enet_host.service(0).unwrap() {
            dbg!(event);
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

    pub fn maintain(&mut self) {
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
