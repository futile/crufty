use ecs::{System, DataHelper, EntityIter};
use ecs::system::EntityProcess;

use super::LevelServices;

use crate::components::LevelComponents;

pub struct VelocitySystem;

impl System for VelocitySystem {
    type Components = LevelComponents;
    type Services = LevelServices;
}

impl EntityProcess for VelocitySystem {
    fn process(&mut self,
               entities: EntityIter<'_, LevelComponents>,
               data: &mut DataHelper<LevelComponents, LevelServices>) {
        for e in entities {
            let velocity = data.velocity[e];

            if let Some(mut position) = data.position.get(&e) {
                data.velocity[e].last_pos = position;

                position.x += velocity.vx;
                position.y += velocity.vy;

                data.position.set(&e, position);
            }

            data.velocity[e].vx = 0.0;
            data.velocity[e].vy = 0.0;
        }
    }
}
