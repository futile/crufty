use ecs::system::EntityProcess;
use ecs::{DataHelper, EntityIter, System};

use super::LevelServices;

use crate::application::InputIntent;
use crate::components::Facing;
use crate::components::LevelComponents;
use crate::game::EntityOps;

use crate::na::{self, Vector2};

use num::traits::Zero;

pub struct MovementSystem;

impl System for MovementSystem {
    type Components = LevelComponents;
    type Services = LevelServices;
}

fn add_clamp_to_zero(val: f32, add: f32, minmax: f32) -> f32 {
    if val < 0.0 {
        na::clamp(val + add, -minmax, 0.0)
    } else {
        na::clamp(val - add, 0.0, minmax)
    }
}

impl EntityProcess for MovementSystem {
    fn process(
        &mut self,
        entities: EntityIter<'_, LevelComponents>,
        data: &mut DataHelper<LevelComponents, LevelServices>,
    ) {
        for e in entities {
            let (move_left, move_right) = {
                let intents = &data.intents[e];

                (
                    intents.contains(&InputIntent::MoveLeft),
                    intents.contains(&InputIntent::MoveRight),
                )
            };

            let delta = data.services.delta_time_s;

            if move_left || move_right {
                if let Some(facing) = data.facing.borrow(&e) {
                    *facing = if move_right {
                        Facing::Right
                    } else {
                        Facing::Left
                    }
                }
            }

            if data.services.collision_world.on_ground(**e) {
                if move_left == move_right {
                    data.play_animation(e.into(), "stand");
                } else {
                    data.play_animation(e.into(), "walk");
                }
            }

            let movement = {
                let movement = &mut data.movement[e];

                if move_left == move_right {
                    if movement.vel.is_zero() {
                        continue;
                    }

                    // TODO reduce speed instead(e.g. by acc)
                    movement.vel.x = add_clamp_to_zero(
                        movement.vel.x,
                        movement.acc.x * delta,
                        movement.max_vel.x,
                    );
                    movement.vel.y = add_clamp_to_zero(
                        movement.vel.y,
                        movement.acc.y * delta,
                        movement.max_vel.y,
                    );
                } else {
                    let acc = if move_left {
                        -movement.acc
                    } else {
                        movement.acc
                    } * delta;

                    let n = &(movement.vel + acc);
                    let min = &-movement.max_vel;
                    let max = &movement.max_vel;

                    let inter_x = na::partial_clamp(&n.x, &min.x, &max.x);
                    let inter_y = na::partial_clamp(&n.y, &min.y, &max.y);

                    // TODO working around this issue: https://github.com/rustsim/nalgebra/issues/401
                    // movement.vel = *inter.unwrap_or(&Vector2::zero());
                    movement.vel = Vector2::new(*inter_x.unwrap_or(&0.0), *inter_y.unwrap_or(&0.0));
                }

                movement.clone()
            };

            let vel = movement.vel;

            let changed_vel = {
                let velocity = &mut data.velocity[e];
                velocity.vx += delta * vel.x;
                velocity.vy += delta * vel.y;

                *velocity
            };

            data.services
                .changed_flags
                .velocity
                .insert(**e, changed_vel);
            data.services.changed_flags.movement.insert(**e, movement);
        }
    }
}
