use ecs::{ System, DataHelper, EntityIter, EntityData, Entity };
use ecs::system::EntityProcess;

use super::LevelServices;

use components::{LevelComponents, CollisionShape};

use na::{self, Pnt2, Iso2, Vec2};
use nc::world::CollisionGroups;
use nc::inspection::Repr;

use std::sync::Arc;
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

impl System for CollisionSystem {
    type Components = LevelComponents;
    type Services = LevelServices;

    fn activated(&mut self, e: &EntityData<Self::Components>, comps: &Self::Components, services: &mut Self::Services) {
        let uid = self.get_free_uid();

        let pos = &comps.position[*e];
        let coll = &comps.collision[*e];

        let shape = match coll.shape {
            CollisionShape::SingleBox(ref cuboid) => cuboid,
            CollisionShape::TwoBoxes{ x: _, y: _ } => unimplemented!(),
        };

        services.collision_world.add(uid,
                                     Iso2::new(Vec2::new(pos.x, pos.y), na::zero()),
                                     Arc::new(Box::new(shape.clone()) as Box<Repr<Pnt2<f32>, Iso2<f32>>>),
                                     CollisionGroups::new(),
                                     ***e);

        self.entity_uids.insert(***e, uid);

        println!("CollisionSystem::activated {}", uid);
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
            let pos = data.position[e];
            let uid = self.entity_uids[&**e];

            data.services.collision_world.defered_set_position(uid, Iso2::new(Vec2::new(pos.x, pos.y), na::zero()),
);
        }

        data.services.collision_world.update();
    }
}
