use ecs::{ System, DataHelper, EntityIter, EntityData, Entity };
use ecs::system::EntityProcess;

use super::LevelServices;

use components::{LevelComponents, Position, Collision, CollisionType};

use na::{self, Iso2, Vec2, Norm};
use nc::world::CollisionGroups;
use num::traits::Zero;

use std::collections::HashMap;

pub struct CollisionSystem {
    next_uid: usize,
    recycled_uids: Vec<usize>,
    entity_uids: HashMap<Entity, usize>,
}

impl CollisionSystem {
    pub fn new() -> CollisionSystem {
        CollisionSystem {
            next_uid: 0,
            recycled_uids: Vec::new(),
            entity_uids: HashMap::new(),
        }
    }

    fn get_free_uid(&mut self) -> usize {
        match self.recycled_uids.pop() {
            Some(uid) => uid,
            None => { self.next_uid += 1; self.next_uid - 1 },
        }
    }

    fn release_uid(&mut self, uid: usize) {
        self.recycled_uids.push(uid);
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct CollisionEntityData {
    pub entity: Entity,
}

fn wpos_to_cpos(pos: &Position, coll: &Collision) -> Vec2<f32> {
    Vec2::new(pos.x / 32.0 + coll.cpos_offset().x, pos.y / 32.0 + coll.cpos_offset().y)
}

impl System for CollisionSystem {
    type Components = LevelComponents;
    type Services = LevelServices;

    fn activated(&mut self, e: &EntityData<Self::Components>, comps: &Self::Components, services: &mut Self::Services) {
        let uid = self.get_free_uid();

        let pos = &comps.position[*e];
        let coll = &comps.collision[*e];

        let data = CollisionEntityData {
            entity: ***e,
        };

        services.collision_world.add(uid,
                                     Iso2::new(wpos_to_cpos(pos, &coll), na::zero()),
                                     // Arc::new(Box::new(shape.clone()) as Box<Repr<Pnt2<f32>, Iso2<f32>>>),
                                     coll.shape().clone(),
                                     CollisionGroups::new(),
                                     data);

        self.entity_uids.insert(***e, uid);

        println!("CollisionSystem::activated {:?}", data);
    }

    fn deactivated(&mut self, e: &EntityData<Self::Components>, _: &Self::Components, services: &mut Self::Services) {
        if let Some(uid) = self.entity_uids.remove(&***e) {
            services.collision_world.remove(uid);
            self.release_uid(uid);

            println!("CollisionSystem::deactivated {}", uid);
        }

        println!("ColisionSystem::deactivated: no uid found for entity");
    }
}

impl EntityProcess for CollisionSystem {
    fn process(&mut self, entities: EntityIter<LevelComponents>, data: &mut DataHelper<LevelComponents, LevelServices>) {
        for e in entities {
            let uid = self.entity_uids[&**e];

            let cpos = wpos_to_cpos(&data.position[e], &data.collision[e]);

            data.services.collision_world.defered_set_position(uid, Iso2::new(cpos, na::zero()),);
        }

        data.services.collision_world.update();

        let mut contacts = Vec::new();

        data.services.collision_world.contacts(|d1, d2, c| {
            // println!("d1: {:?}, d2: {:?}, c: {:?}", d1, d2, c);
            contacts.push((d1.clone(), d2.clone(), c.clone()));
        });

        for contact in &contacts {
            let en1 = &contact.0.entity;
            let en2 = &contact.1.entity;
            let c   = &contact.2;

            if c.depth <= 0.0 {
                continue;
            }

            let (pos1, ovel1, ct1) = data.with_entity_data(en1, | en, comps | { (comps.position[en], comps.velocity.get(&en), comps.collision[en].collision_type()) }).unwrap();
            let (pos2, ovel2, ct2) = data.with_entity_data(en2, | en, comps | { (comps.position[en], comps.velocity.get(&en), comps.collision[en].collision_type()) }).unwrap();

            if ct1 == CollisionType::Solid || ct2 == CollisionType::Solid {
                let vel1 = match ovel1 {
                    Some(ref v) => Vec2::new(pos1.x - v.last_pos.x, pos1.y - v.last_pos.y),
                    None => na::zero(),
                };
                let vel2 = match ovel2 {
                    Some(ref v) => Vec2::new(pos2.x - v.last_pos.x, pos2.y - v.last_pos.y),
                    None => na::zero(),
                };

                let (f1, f2) = if vel1.is_zero() && vel2.is_zero() {
                    (1.0, 1.0)
                } else {
                    let sum_len = (vel1 + vel2).norm();
                    (vel1.norm() / sum_len, vel2.norm() / sum_len)
                };

                if f1 > 0.0 {
                    data.with_entity_data(en1, |en, comps| {
                        let pos = &mut comps.position[en];

                        pos.y -= c.normal.y * c.depth * f1 * 32.0;
                        pos.x -= c.normal.x * c.depth * f1 * 32.0;
                    });
                }
                if f2 > 0.0 {
                    data.with_entity_data(en2, |en, comps| {
                        let pos = &mut comps.position[en];

                        pos.y -= c.normal.y * c.depth * f2 * 32.0;
                        pos.x -= c.normal.x * c.depth * f2 * 32.0;
                    });
                }
            }
        }
    }
}
