extern crate crufty;
extern crate specs;

use specs::Join;

use crufty::v2::{self, CContext, Position, Info, WorldSyncer};

struct PositionSystem;

impl specs::System<CContext> for PositionSystem {
    fn run(&mut self, arg: specs::RunArg, context: CContext) {
        let (ents, mut poss) = arg.fetch(|w| {
           (w.entities(), w.write::<Position>()) 
        });

        let mut pos_updates = context.positions.lock().unwrap();
        for (ent, pos) in (&ents, &mut poss).iter() {
            pos.x = (pos.x + 1.0) % 20.0;
            pos_updates.insert(ent, *pos);
        }
    }
}

fn main() {
    let mut world = specs::World::new();
    world.register::<Position>();
    // world.add_resource(Info(42));
    world.create_now().with(Position {x: 3.0, y: 4.0}).build();

    let mut p = specs::Planner::<CContext>::new(world, 1);
    p.add_system(PositionSystem, "PositionSystem", 0);

    let context = CContext::default();

    p.dispatch(context.clone());

    let ser = v2::serialize_ccontext(&context);
    println!("ser.len(): {}", ser.len());

    let mut world2 = specs::World::new();
    world2.register::<Position>();
    let mut p2 = specs::Planner::<CContext>::new(world2, 1);

    let mut ws = WorldSyncer::default();

    ws.deserialize_into_world(p2.mut_world(), &ser);

    p2.run_custom(|arg| {
        let (ents, poss) = arg.fetch(|w| {
            (w.entities(), w.read::<Position>())
        });

        for (e, p) in (&ents, &poss).iter() {
            println!("e: {:?}, pos: {:?}", e, p);
        }
    });
}
