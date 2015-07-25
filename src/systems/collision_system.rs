use ecs::{ System, DataHelper, EntityIter, EntityData };
use ecs::system::EntityProcess;

use super::LevelServices;

use components::{LevelComponents};

pub struct CollisionSystem;

impl System for CollisionSystem {
    type Components = LevelComponents;
    type Services = LevelServices;

    fn activated(&mut self, _: &EntityData<Self::Components>, comps: &Self::Components, services: &mut Self::Services) {
        println!("CollisionSystem::activated {}", services.delta_time_s);
    }
}

impl EntityProcess for CollisionSystem {
    fn process(&mut self, entities: EntityIter<LevelComponents>, data: &mut DataHelper<LevelComponents, LevelServices>) {
        //
    }
}
