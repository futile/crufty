use ecs::{System, DataHelper, EntityIter};
use ecs::system::EntityProcess;

use super::LevelServices;

use crate::components::LevelComponents;
use crate::components::{Jump, JumpState};
use crate::application::InputIntent;
use crate::game::{EntityOps};

use crate::na::Vector2;

use num::traits::Zero;

pub struct JumpSystem;

impl System for JumpSystem {
    type Components = LevelComponents;
    type Services = LevelServices;
}

const JUMP_RISE_TIME_S: f32 = 0.5;
lazy_static! {
    static ref JUMP_RISE_VEL: Vector2<f32> = Vector2::new(0.0, 150.0);
}

impl EntityProcess for JumpSystem {
    fn process(&mut self,
               entities: EntityIter<'_, LevelComponents>,
               data: &mut DataHelper<LevelComponents, LevelServices>) {
        let delta = data.services.delta_time_s;
        let g = data.services.gravity;

        for e in entities {
            let do_jump = data.intents[e].contains(&InputIntent::Jump);
            let mut jump: Jump = data.jump[e];

            match jump.state {
                JumpState::Idle => {
                    if !do_jump {
                        continue;
                    }

                    if !data.services.collision_world.on_ground(**e) {
                        continue;
                    }

                    jump.state = JumpState::Rising;
                    jump.jump_time_remaining = JUMP_RISE_TIME_S;

                    data.play_animation(e.into(), "jump");
                }
                s @ JumpState::Rising |
                s @ JumpState::MidairIdle => {
                    jump.jump_time_remaining -= delta;
                    if jump.jump_time_remaining <= 0.0 {
                        jump.state = JumpState::Idle;
                    }
                    if s == JumpState::Rising && !do_jump {
                        jump.state = JumpState::MidairIdle;
                        data.play_animation(e.into(), "stand");
                    }
                }
            }

            // drop mutability
            let jump = jump;
            data.jump[e] = jump;

            let vel_change: Vector2<f32> = {
                let get_antigrav_vel = || if let Some(gravity) = data.gravity.get(&e) {
                    Vector2::new(0.0, g * gravity.f)
                } else {
                    Vector2::zero()
                };

                match jump.state {
                    JumpState::Rising => *JUMP_RISE_VEL + get_antigrav_vel(),
                    JumpState::MidairIdle if do_jump => get_antigrav_vel() / 2.0,
                    JumpState::MidairIdle | JumpState::Idle => Vector2::zero(),
                }
            };

            let velocity = &mut data.velocity[e];
            velocity.vx += delta * vel_change.x;
            velocity.vy += delta * vel_change.y;
        }
    }
}
