use std::time::{Duration, Instant};

use specs::{self, Join};

use util::State;
// use net::udp::{self, CongestionControl, CongestionStatus, ReliabilityWrapper, ReceiveEvent};
use v2::{self, CContext, Position, Info, WorldSyncer};

use server::net::NetworkInterface;
use server::ServerTransition;

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
    niface: NetworkInterface,
}

impl ServerRunState {
    pub fn new(niface: NetworkInterface) -> ServerRunState {
        ServerRunState { niface: niface }
    }
}

impl State<ServerTransition> for ServerRunState {
    fn run(self: Box<Self>) -> ServerTransition {
        let mut world = specs::World::new();
        world.register::<Position>();
        // world.add_resource(Info(42));
        for i in 0..500 {
            world.create_now().with(Position { x: 3.0, y: i as f32 }).build();
        }

        let mut p = specs::Planner::<CContext>::new(world, 1);
        p.add_system(PositionSystem, "PositionSystem", 0);

        let context = CContext::default();

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

            // 2. send updates
            self.niface.perform_send_phase(&context).unwrap();

            // 3. FIXME debug render

            // 4. receive blocking, deadline is min(next_sim, next_send)
            // let timeout_simulation = update_interval.checked_sub(previous_update.elapsed());
            // let timeout_send = congestion_control.congestion_status().wait_time();
            let min_timeout = Duration::new(0, 1);
            // let timeout = match (timeout_simulation, timeout_send) {
            //     (Some(tsim), Some(tsend)) => ::std::cmp::min(tsim, tsend),
            //     _ => min_timeout, // TODO instead of minimum timeout, just skip receiving maybe?
            // };
            let timeout = update_interval.checked_sub(previous_update.elapsed()).unwrap_or(min_timeout);

            self.niface.perform_receive_phase(Some(timeout));
        }

        ServerTransition::Shutdown
    }
}
