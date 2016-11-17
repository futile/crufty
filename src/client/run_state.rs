use specs::{self, Join};

use util::State;
use net::udp::{self, UdpConnection, ReceiveEvent};
use v2::{self, CContext, Position, Info, WorldSyncer};

use super::ClientTransition;

pub struct ClientRunState {
    conn: UdpConnection,
}

impl ClientRunState {
    pub fn new(conn: UdpConnection) -> ClientRunState {
        ClientRunState { conn: conn }
    }
}

impl State<ClientTransition> for ClientRunState {
    fn run(mut self: Box<Self>) -> ClientTransition {
        let mut world = specs::World::new();
        world.register::<Position>();

        let mut planner = specs::Planner::<CContext>::new(world, 1);
        let mut syncer = WorldSyncer::default();

        // game loop:
        loop {
            // 1. receive
            self.conn.recv_with_timeout(None, |e| match e {
                ReceiveEvent::NewAck(_mid, _rtt) => {
                    // println!("new ack; id: {:?}, rtt: {}ms", _mid, udp::dur_to_ms(&_rtt))
                }
                ReceiveEvent::NewData(data) => {
                    syncer.deserialize_into_world(planner.mut_world(), data);
                }
            });

            // for ACKs
            self.conn.send_bytes(&[]).unwrap();

            // 2. render aka println! (with timestamp)
            planner.run_custom(|arg| {
                let (ents, poss) = arg.fetch(|w| (w.entities(), w.read::<Position>()));

                for (e, p) in (&ents, &poss).iter() {
                    println!("e: {:?}, pos: {:?}", e, p);
                }
            });

            self.conn.check_for_timeouts(|msg_id| println!("msg {:?} timed out", msg_id));
        }
    }
}
