pub use self::animation::{Animation, SpriteSheet};
use self::events::EventReceiver;
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

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Interaction {
    WarpInRoom { x: f32, y: f32 },
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
        call: F,
    ) -> Option<R>
    where
        F: FnOnce(EntityData<LevelComponents>, &mut LevelComponents) -> R,
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

        if let Some(Some(changed_ssa)) = eod.with_entity_data(self, move |en, comps| {
            let ssa = match comps.sprite_sheet_animation.borrow(&en) {
                Some(ssa) => ssa,
                _ => return None,
            };

            ssa.current_frame = 0;
            ssa.frame_time_remaining = anim.frame_durations[0];
            ssa.animation = anim.clone();

            Some(ssa.clone())
        }) {
            self.services
                .changed_flags
                .sprite_sheet_animation
                .insert(eod.as_entity(), changed_ssa);
        }
    }

    fn move_entity(&mut self, eod: EntityOrData, new_pos: &Position, warp: bool) {
        let (mut coll_shape, last_pos) = match eod.with_entity_data(self, |en, comps| {
            let coll_shape = comps.collision_shape[en].clone();
            let last_pos = comps.velocity.borrow(&en).map(|vel| vel.last_pos);

            (coll_shape, last_pos)
        }) {
            Some(x) => x,
            None => return,
        };

        let mut colls = SmallVec::<[crate::util::collision_world::Collision; 2]>::new();

        for coll in &colls {
            assert_eq!(coll.collider, eod.as_entity());
        }

        let resolved_pos = self.services.collision_world.move_entity(
            eod.as_entity(),
            &coll_shape,
            &new_pos,
            if warp { None } else { last_pos.as_ref() },
            &mut colls,
        );

        let mut pos_changed = true;
        if let Some(last_pos) = last_pos {
            if resolved_pos == last_pos {
                pos_changed = false;
            }
        }

        if pos_changed {
            eod.with_entity_data(self, move |en, comps| {
                comps.position[en] = resolved_pos;
            });

            self.services
                .changed_flags
                .position
                .insert(eod.as_entity(), resolved_pos);
        }

        use self::events::CollisionEnded;
        let mut ended: SmallVec<[CollisionEnded; 2]> = SmallVec::new();

        coll_shape.ongoing_collisions.others.retain(|&mut other| {
            if !colls.iter().any(|cl| cl.collided == other) {
                ended.push(CollisionEnded {
                    collider: eod.as_entity(),
                    collided: other,
                });
                false
            } else {
                true
            }
        });

        use self::events::CollisionStarted;
        let started: SmallVec<[CollisionStarted; 2]> = colls
            .iter()
            .filter_map(|cl| {
                if coll_shape.ongoing_collisions.add(cl.collided) {
                    Some(CollisionStarted {
                        collider: cl.collider,
                        collided: cl.collided,
                    })
                } else {
                    None
                }
            })
            .collect();

        eod.with_entity_data(self, |en, comps| {
            comps.collision_shape[en] = coll_shape.clone();
        });

        self.services
            .changed_flags
            .collision_shape
            .insert(eod.as_entity(), coll_shape);

        for event in ended {
            self.receive_event(event);
        }
        for event in started {
            self.receive_event(event);
        }
    }
}
