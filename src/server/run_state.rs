use std::time::{Duration, Instant};

use specs::{self, Join};

use util::State;
use net::udp::{self, CongestionControl, CongestionStatus, UdpConnection, ReceiveEvent};
use v2::{self, CContext, Position, Info, WorldSyncer};

use super::ServerTransition;

struct PositionSystem;

impl specs::System<CContext> for PositionSystem {
    fn run(&mut self, arg: specs::RunArg, context: CContext) {
        let (ents, mut poss) = arg.fetch(|w| (w.entities(), w.write::<Position>()));

        let mut pos_updates = context.positions.lock().unwrap();
        for (ent, pos) in (&ents, &mut poss).iter() {
            pos.x = (pos.x + 1.0) % 20.0;
            pos_updates.insert(ent, *pos);
        }
    }
}

pub struct ServerRunState {
    conn: UdpConnection,
}

impl ServerRunState {
    pub fn new(conn: UdpConnection) -> ServerRunState {
        ServerRunState { conn: conn }
    }
}

impl State<ServerTransition> for ServerRunState {
    fn run(self: Box<Self>) -> ServerTransition {
        let mut world = specs::World::new();
        world.register::<Position>();
        // world.add_resource(Info(42));
        world.create_now().with(Position { x: 3.0, y: 4.0 }).build();

        let mut p = specs::Planner::<CContext>::new(world, 1);
        p.add_system(PositionSystem, "PositionSystem", 0);

        let context = CContext::default();

        let mut conn = CongestionControl::new(self.conn);

        let update_interval = Duration::from_secs(1) / 60;

        let mut previous_update = Instant::now();
        let mut lag_behind_simulation = Duration::new(0, 0);

        // game loop:
        loop {
            // 1. catch up simulation
            let current = Instant::now();
            let elapsed = current.duration_since(previous_update);
            previous_update = current;
            lag_behind_simulation += elapsed;

            let mut update_counter = 0;
            while lag_behind_simulation >= update_interval {
                // simulation updated **here**
                p.dispatch(context.clone());

                lag_behind_simulation -= update_interval;
                update_counter += 1;
            }
            println!("updated {} times", update_counter);

            // 2. send (if rate allows)
            if let CongestionStatus::ReadyToSend = conn.congestion_status() {
                let ser = v2::serialize_ccontext(&context);
                conn.send_bytes(&ser).unwrap();
                println!("update sent");
            }

            // 3. FIXME debug render

            // 4. receive blocking, deadline is min(next_sim, next_send)
            let timeout_simulation = update_interval.checked_sub(previous_update.elapsed());
            let timeout_send = conn.congestion_status().wait_time();
            let min_timeout = Duration::new(0, 1);
            let timeout = match (timeout_simulation, timeout_send) {
                (Some(tsim), Some(tsend)) => ::std::cmp::min(tsim, tsend),
                _ => min_timeout, // TODO instead of minimum timeout, just skip receiving maybe?
            };

            conn.recv_with_timeout(Some(timeout), |e| match e {
                ReceiveEvent::NewAck(_msg_id, _rtt) => {
                    // println!("ack; id: {:?}, rtt: {:?}ms", _msg_id, udp::dur_to_ms(&_rtt));
                }
                ReceiveEvent::NewData(data) => {
                    if data.len() > 0 {
                        println!("received msg(unexpected?): {:?}", data);
                    }
                }
            });

            conn.check_for_timeouts(|msg_id| println!("msg {:?} timed out", msg_id));
        }

        ServerTransition::Shutdown
    }
}
