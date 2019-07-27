use ecs::system::EntityProcess;
use ecs::{DataHelper, EntityIter, System};

use super::LevelServices;

use crate::components::LevelComponents;
use crate::components::SpriteSheetAnimation;

#[derive(Default, Debug)]
pub struct SpriteSheetAnimationSystem;

impl SpriteSheetAnimationSystem {
    pub fn new() -> SpriteSheetAnimationSystem {
        Default::default()
    }
}

impl System for SpriteSheetAnimationSystem {
    type Components = LevelComponents;
    type Services = LevelServices;
}

impl EntityProcess for SpriteSheetAnimationSystem {
    fn process(
        &mut self,
        entities: EntityIter<'_, LevelComponents>,
        data: &mut DataHelper<LevelComponents, LevelServices>,
    ) {
        let delta_s = data.services.delta_time_s;

        for e in entities {
            let mut changed = false;

            let new_si = {
                let ssa: &mut SpriteSheetAnimation = &mut data.sprite_sheet_animation[e];

                ssa.frame_time_remaining -= delta_s;

                if ssa.frame_time_remaining < 0.0 {
                    ssa.current_frame = (ssa.current_frame + 1) % ssa.animation.num_frames;
                    ssa.frame_time_remaining +=
                        ssa.animation.frame_durations[ssa.current_frame as usize];
                    changed = true;
                }

                ssa.animation.create_sprite_info(ssa.current_frame)
            };

            data.sprite[e].info = new_si;

            if changed {
                data.services
                    .changed_flags
                    .sprite
                    .insert(**e, data.sprite[e].clone());
            }
        }
    }
}
