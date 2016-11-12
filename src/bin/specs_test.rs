extern crate specs;

use specs::Join;

#[derive(Copy,Clone,Debug)]
struct Position {
    x: f32,
    y: f32,
}

impl specs::Component for Position {
    type Storage = specs::VecStorage<Position>;
}

#[derive(Clone,Copy,Debug)]
struct Info(i32);

fn main() {
    let mut world = specs::World::new();

    world.register::<Position>();

    world.add_resource(Info(42));

    let e = world.create_now().with(Position {x: 3.0, y: 4.0}).build();

    let mut p = specs::Planner::<()>::new(world, 4);

    p.run_custom(|arg| {
        let (mut poss, mut info) = arg.fetch(|world| {
            (world.write::<Position>(), world.write_resource::<Info>())
        });

        for pos in (&mut poss).iter() {
            pos.x += 1.0;
            pos.y += 2.0;
        }

        info.0 += 7;
    });

    p.wait();

    p.run1w0r(|pos: &mut Position| {
        pos.y = 0.0;
    });

    p.run_custom(|arg| {
        let (ents, poss, info) = arg.fetch(|world| {
            (world.entities(), world.read::<Position>(), world.read_resource::<Info>())
        });

        for (eid, pos) in (&ents, &poss).iter() {
            println!("{:?}: {:?}", eid, pos);
        }

        println!("info: {:?}", *info);
    });

    p.wait();
}
