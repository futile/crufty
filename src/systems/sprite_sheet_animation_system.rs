use ecs::{System, DataHelper, EntityIter};
use ecs::system::EntityProcess;

use super::LevelServices;

use components::LevelComponents;
use components::SpriteSheetAnimation;

pub struct SpriteSheetAnimationSystem;

impl SpriteSheetAnimationSystem {
    pub fn new() -> SpriteSheetAnimationSystem {
        SpriteSheetAnimationSystem
    }
}

impl System for SpriteSheetAnimationSystem {
    type Components = LevelComponents;
    type Services = LevelServices;
}

impl EntityProcess for SpriteSheetAnimationSystem {
    fn process(&mut self,
               entities: EntityIter<LevelComponents>,
               data: &mut DataHelper<LevelComponents, LevelServices>) {
        let delta_s = data.services.delta_time_s;

        for e in entities {
            let new_si = {
                let ssa: &mut SpriteSheetAnimation = &mut data.sprite_sheet_animation[e];

                ssa.frame_time_remaining -= delta_s;

                if ssa.frame_time_remaining < 0.0 {
                    ssa.current_frame = (ssa.current_frame + 1) % ssa.animation.num_frames;
                    ssa.frame_time_remaining +=
                        ssa.animation.frame_durations[ssa.current_frame as usize];
                }

                ssa.animation.create_sprite_info(ssa.current_frame)
            };

            data.sprite_info[e] = new_si;
        }
    }
}
