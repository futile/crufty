pub use self::animation::{Animation, SpriteSheet};
pub use self::resource_store::*;

use ecs::{DataHelper, Entity, EntityData};

use crate::components::{LevelComponents, Position};
use crate::systems::LevelServices;

use smallvec::SmallVec;

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

// *can* use `Either` when this issue is done: https://github.com/bluss/either/issues/33
// But we might not want to, as this might be simple enough (but it's so much boilerplate :()
// -> Issue might never be done due to `L != R` not specifiable
pub enum EntityOrData<'a> {
    Entity(Entity),
    EntityData(EntityData<'a, LevelComponents>),
}

impl From<Entity> for EntityOrData<'_> {
    fn from(e: Entity) -> EntityOrData<'static> {
        EntityOrData::Entity(e)
    }
}

impl<'a> From<EntityData<'a, LevelComponents>> for EntityOrData<'a> {
    fn from(ed: EntityData<'a, LevelComponents>) -> EntityOrData<'a> {
        EntityOrData::EntityData(ed)
    }
}

impl EntityOrData<'_> {
    fn with_entity_data<F, R>(
        &self,
        data: &mut DataHelper<LevelComponents, LevelServices>,
        mut call: F,
    ) -> Option<R>
    where
        F: FnMut(EntityData<LevelComponents>, &mut LevelComponents) -> R,
    {
        match *self {
            EntityOrData::Entity(e) => data.with_entity_data(&e, call),
            EntityOrData::EntityData(ed) => Some(call(ed, &mut data.components)),
        }
    }

    fn as_entity(&self) -> Entity {
        match *self {
            EntityOrData::Entity(e) => e,
            EntityOrData::EntityData(ed) => **ed,
        }
    }
}

pub trait EntityOps {
    /// play animation with name `anim_name` for entity `e`
    fn play_animation(&mut self, eod: EntityOrData, anim_name: &str);

    fn move_entity(&mut self, eod: EntityOrData, new_pos: &Position, warp: bool);
}

impl EntityOps for DataHelper<LevelComponents, LevelServices> {
    fn play_animation(&mut self, eod: EntityOrData, anim_name: &str) {
        // TODO this whole thing could be nicer, but ecs-rs currently doesn't let us.
        // especially having to clone `anim` twice is annoying. but it's infrequent code,
        // so we don't care for now.

        let ss_handle = eod.with_entity_data(self, |en, comps| {
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

        let anim = match self
            .services
            .resource_store
            .get_sprite_sheet(ss_handle)
            .get(anim_name)
        {
            Some(anim) => anim.clone(),
            _ => return,
        };

        eod.with_entity_data(self, move |en, comps| {
            let ssa = match comps.sprite_sheet_animation.borrow(&en) {
                Some(ssa) => ssa,
                _ => return,
            };

            ssa.current_frame = 0;
            ssa.frame_time_remaining = anim.frame_durations[0];
            ssa.animation = anim.clone();
        });
    }

    fn move_entity(&mut self, eod: EntityOrData, new_pos: &Position, warp: bool) {
        let (coll, last_pos) = match eod.with_entity_data(self, |en, comps| {
            let coll = comps.collision_shape[en].clone();
            let last_pos = match warp {
                true => None,
                false => comps.velocity.borrow(&en).map(|vel| vel.last_pos),
            };

            (coll, last_pos)
        }) {
            Some(x) => x,
            None => return,
        };

        let mut colls = SmallVec::<[crate::util::collision_world::Collision; 2]>::new();

        let resolved_pos = self.services.collision_world.move_entity(
            eod.as_entity(),
            &coll,
            &new_pos,
            last_pos.as_ref(),
            &mut colls,
        );

        eod.with_entity_data(self, move |en, comps| {
            comps.position[en] = resolved_pos;
        });

        for coll in colls {
            use self::events::{CollisionStarted, EventReceiver};
            self.receive_event(CollisionStarted {
                collider: coll.collider,
                collided: coll.collided,
            });
        }
    }
}
