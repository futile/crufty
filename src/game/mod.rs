pub use self::animation::{Animation, SpriteSheet};
pub use self::resource_store::*;

use ecs::{DataHelper, Entity};

use components::LevelComponents;
use systems::LevelServices;

mod animation;
mod resource_store;

pub mod events;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct PlayerId(u16);

impl From<u16> for PlayerId {
    fn from(val: u16) -> PlayerId {
        PlayerId(val)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Interaction {
    Warp,
}

pub trait EntityOps {
    fn play_animation(&mut self, e: Entity, anim_name: &str);
}

impl EntityOps for DataHelper<LevelComponents, LevelServices> {
    fn play_animation(&mut self, e: Entity, anim_name: &str) {
        // TODO this whole thing could be nicer, but ecs-rs currently doesn't let us.
        // especially having to clone `anim` twice is annoying. but it's infrequent code,
        // so we don't care for now.

        let ss_handle = self.with_entity_data(&e, |en, comps| {
            comps.sprite_sheet_animation.borrow(&en).and_then(|ssa| {
                // if the animation is already running, don't restart it
                if &ssa.animation.name[..] == anim_name {
                    None
                } else {
                    Some(ssa.sheet_handle)
                }
            })
        });

        let ss_handle = match ss_handle {
            Some(Some(handle)) => handle,
            _ => return,
        };

        let anim = match self.services.resource_store.get_sprite_sheet(ss_handle).get(anim_name) {
            Some(anim) => anim.clone(),
            _ => return,
        };

        self.with_entity_data(&e, move |en, comps| {
            let ssa = match comps.sprite_sheet_animation.borrow(&en) {
                Some(ssa) => ssa,
                _ => return,
            };

            ssa.current_frame = 0;
            ssa.frame_time_remaining = anim.frame_durations[0];
            ssa.animation = anim.clone();
        });
    }
}
