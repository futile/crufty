use ecs::{ System, DataHelper, EntityIter, EntityData, Entity };
use ecs::system::InteractProcess;

use super::LevelServices;

use components::{LevelComponents, Position, Collision, CollisionType};

pub struct CollisionSystem;

impl CollisionSystem {
    pub fn new() -> CollisionSystem {
        CollisionSystem
    }
}

impl System for CollisionSystem {
    type Components = LevelComponents;
    type Services = LevelServices;
}

impl InteractProcess for CollisionSystem {
    fn process(&mut self, dynamic_entities: EntityIter<LevelComponents>, static_entities_iter: EntityIter<LevelComponents>, data: &mut DataHelper<LevelComponents, LevelServices>) {
        let static_entities: Vec<_>= static_entities_iter.collect();

        for e1 in dynamic_entities {
            let p1 = &data.position[e1];
            let c1 = &data.collision[e1];

            for e2 in &static_entities {
                let p2 = &data.position[*e2];
                let c2 = &data.collision[*e2];

                // 1. detect if collision
                // let collision_x = 

                // 2. if both SOLID, resolve collision
            }
        }
    }
}
