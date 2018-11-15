use std::collections::HashMap;
use std::intrinsics::type_id;
use std::io::Write;
use std::net::Ipv4Addr;
use std::time::Instant;

use bincode::serialize_into;
use ecs::{Entity, World};

use enet::{self, Event, Packet, PacketMode, PeerState};

use super::{ENET, PORT, RESEND_DURATION, UPDATE_CHANNEL_ID};
use crate::components::*;
use crate::systems::LevelSystems;

trait UpdateMapFuncs {
    fn serialize_into(&mut self, out: &mut impl Write);
}

type UpdateMap<C> = HashMap<Entity, (C, u64, Instant)>;

impl<C> UpdateMapFuncs for UpdateMap<C>
where
    C: 'static + serde::Serialize,
{
    fn serialize_into(&mut self, mut out: &mut impl Write) {
        let now = Instant::now();

        let mut tag_written = false;

        // TODO remove drain() and instead send responses from client
        for (e, mut update) in self.drain() {
            if update.2 > now {
                continue;
            }

            update.2 = now + RESEND_DURATION;

            if !tag_written {
                tag_written = true;
                serialize_into(&mut out, unsafe { &type_id::<C>() }).unwrap();
            } else {
                serialize_into(&mut out, &true).unwrap();
            }

            serialize_into(&mut out, &e.id()).unwrap();
            serialize_into(&mut out, &update.1).unwrap();
            serialize_into(&mut out, &update.0).unwrap();
        }

        if tag_written {
            serialize_into(&mut out, &false).unwrap();
        }
    }
}

#[derive(Debug, Default)]
struct PeerData {
    position: UpdateMap<Position>,
    camera: UpdateMap<Camera>,
    velocity: UpdateMap<Velocity>,
    jump: UpdateMap<Jump>,
    gravity: UpdateMap<Gravity>,
    facing: UpdateMap<Facing>,
    intents: UpdateMap<Intents>,
    interactor: UpdateMap<Interactor>,
}

macro_rules! new_from_world_inner {
    ($name:ident, $res:ident, $world:ident, $en:ident, $sim_time:ident, $now: ident) => {
        if let Some(c) = $world.$name.get(&$en) {
            $res.$name.insert(**$en, (c, $sim_time, $now));
        }
    };
}

macro_rules! update_from_changes_inner {
    ($name:ident, $self:ident, $world:ident, $sim_time:ident, $now: ident) => {
        for (e, c) in $world.services.changed_flags.$name.drain() {
            $self.$name.insert(e, (c, $sim_time, $now));
        }
    };
}

impl PeerData {
    fn new_from_world(world: &mut World<LevelSystems>) -> PeerData {
        let mut res = PeerData::default();

        let sim_time = world.services.simulation_time;
        let now = Instant::now();
        for en in world.entities() {
            new_from_world_inner!(position  , res, world, en, sim_time, now);
            new_from_world_inner!(camera    , res, world, en, sim_time, now);
            new_from_world_inner!(jump      , res, world, en, sim_time, now);
            new_from_world_inner!(gravity   , res, world, en, sim_time, now);
            new_from_world_inner!(facing    , res, world, en, sim_time, now);
            new_from_world_inner!(intents   , res, world, en, sim_time, now);
            new_from_world_inner!(interactor, res, world, en, sim_time, now);
        }

        return res;
    }

    fn update_from_changes(&mut self, world: &mut World<LevelSystems>) {
        let sim_time = world.services.simulation_time;
        let now = Instant::now();
        update_from_changes_inner!(position  , self, world, sim_time, now);
        update_from_changes_inner!(camera    , self, world, sim_time, now);
        update_from_changes_inner!(velocity  , self, world, sim_time, now);
        update_from_changes_inner!(jump      , self, world, sim_time, now);
        update_from_changes_inner!(gravity   , self, world, sim_time, now);
        update_from_changes_inner!(facing    , self, world, sim_time, now);
        update_from_changes_inner!(intents   , self, world, sim_time, now);
        update_from_changes_inner!(interactor, self, world, sim_time, now);
    }

    fn serialize_updates(&mut self) -> Option<Vec<u8>> {
        let mut data = vec![];

        self.position.serialize_into(&mut data);
        self.camera.serialize_into(&mut data);
        self.velocity.serialize_into(&mut data);
        self.jump.serialize_into(&mut data);
        self.gravity.serialize_into(&mut data);
        self.facing.serialize_into(&mut data);
        self.intents.serialize_into(&mut data);
        self.interactor.serialize_into(&mut data);

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
                peer.send_packet(
                    Packet::new(&update_data, PacketMode::UnreliableUnsequenced).unwrap(),
                    UPDATE_CHANNEL_ID,
                )
                .unwrap();
            }
        }
    }
}
