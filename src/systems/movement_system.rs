use ecs::{ System, DataHelper, EntityIter };
use ecs::system::EntityProcess;

use super::LevelServices;

use components::{LevelComponents};

use na::{self, Vec2};

use num::traits::Zero;

pub struct MovementSystem;

impl System for MovementSystem {
    type Components = LevelComponents;
    type Services = LevelServices;
}

impl EntityProcess for MovementSystem {
    fn process(&mut self, entities: EntityIter<LevelComponents>, data: &mut DataHelper<LevelComponents, LevelServices>) {
        for e in entities {
            let vel = {
                let movement = &mut data.movement[e];

                if movement.moving_left == movement.moving_right {
                    // TODO reduce speed instead(e.g. by acc)
                    movement.vel = Vec2::zero();

                    continue;
                }

                let acc = if movement.moving_left {
                    -movement.acc
                } else {
                    movement.acc
                };

                movement.vel = na::partial_clamp(&(movement.vel + acc), &-movement.max_vel, &movement.max_vel).unwrap_or(&Vec2::zero()).clone();

                movement.vel.clone()
            };

            {
                let velocity = &mut data.velocity[e];
                velocity.vx += vel.x;
                velocity.vy += vel.y;
            }
        }
    }
}
