use ecs::{EntityData, System, DataHelper, EntityIter};
use ecs::system::InteractProcess;

use super::LevelServices;

use components::LevelComponents;

pub struct CollisionSystem;

impl CollisionSystem {
    pub fn new() -> CollisionSystem {
        CollisionSystem
    }
}

impl System for CollisionSystem {
    type Components = LevelComponents;
    type Services = LevelServices;

    fn activated(&mut self,
                 e: &EntityData<Self::Components>,
                 data: &Self::Components,
                 services: &mut Self::Services) {
        // TODO `&data.collision[*e]` causes a clone, find a way which doesn't
        services.collision_world.add(***e, &data.collision[*e], &data.position[*e]);
    }

    fn deactivated(&mut self,
                   e: &EntityData<Self::Components>,
                   _data: &Self::Components,
                   services: &mut Self::Services) {
        // TODO remove from tree + drop leaf
        services.collision_world.remove(***e);
    }
}

impl InteractProcess for CollisionSystem {
    fn process(&mut self,
               dynamic_entities: EntityIter<LevelComponents>,
               _: EntityIter<LevelComponents>,
               data: &mut DataHelper<LevelComponents, LevelServices>) {

        for e1 in dynamic_entities {
            let p1 = data.position[e1];
            let v1 = data.velocity[e1];
            let c1 = data.collision[e1].clone();

            data.position[e1] = data.services.collision_world.move_entity(**e1, &c1, &p1, &v1.last_pos);
        }
    }
}
