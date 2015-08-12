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
    fn process(&mut self, dynamic_entities: EntityIter<LevelComponents>, static_entities: EntityIter<LevelComponents>, data: &mut DataHelper<LevelComponents, LevelServices>) {
        }
}
