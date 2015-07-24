use ecs::{ System, DataHelper, EntityIter };
use ecs::system::EntityProcess;

use super::LevelServices;

use components::{LevelComponents};

pub struct VelocitySystem;

impl System for VelocitySystem {
    type Components = LevelComponents;
    type Services = LevelServices;
}

impl EntityProcess for VelocitySystem {
    fn process(&mut self, entities: EntityIter<LevelComponents>, data: &mut DataHelper<LevelComponents, LevelServices>) {
        for e in entities {
            let velocity = data.velocity[e];
            let delta = data.services.delta_time_s;

            if let Some(position) = data.position.borrow(&e) {
                position.x += velocity.vx * delta;
                position.y += velocity.vy * delta;
            }

            data.velocity[e].vx = 0.0;
            data.velocity[e].vy = 0.0;
        }
    }
}
