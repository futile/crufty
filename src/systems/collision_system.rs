use ecs::system::EntityProcess;
use ecs::{DataHelper, EntityData, EntityIter, System};

use super::LevelServices;

use crate::components::LevelComponents;

pub struct CollisionSystem;

impl CollisionSystem {
    pub fn new() -> CollisionSystem {
        CollisionSystem
    }
}

impl System for CollisionSystem {
    type Components = LevelComponents;
    type Services = LevelServices;

    fn activated(
        &mut self,
        e: &EntityData<'_, Self::Components>,
        data: &Self::Components,
        services: &mut Self::Services,
    ) {
        // TODO `&data.collision[*e]` causes a clone, find a way which doesn't
        services
            .collision_world
            .add(***e, &data.collision[*e], &data.position[*e]);
    }

    fn deactivated(
        &mut self,
        e: &EntityData<'_, Self::Components>,
        _data: &Self::Components,
        services: &mut Self::Services,
    ) {
        services.collision_world.remove(***e);
    }
}

impl EntityProcess for CollisionSystem {
    fn process(
        &mut self,
        _dynamic_entities: EntityIter<'_, LevelComponents>,
        _data: &mut DataHelper<LevelComponents, LevelServices>,
    ) {
    }
}
