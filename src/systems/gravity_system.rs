use ecs::{ System, DataHelper, EntityIter };
use ecs::system::EntityProcess;

use super::LevelServices;

use components::{LevelComponents};

pub struct GravitySystem {
    pub g: f32,
}

impl System for GravitySystem {
    type Components = LevelComponents;
    type Services = LevelServices;
}

impl EntityProcess for GravitySystem {
    fn process(&mut self, entities: EntityIter<LevelComponents>, data: &mut DataHelper<LevelComponents, LevelServices>) {
        for e in entities {
            let gravity = data.gravity[e];
            let delta = data.services.delta_time_s;
            let velocity = &mut data.velocity[e];

            velocity.vy -= self.g * gravity.f * delta;
        }
    }
}
