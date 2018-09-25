use ecs::system::EntityProcess;
use ecs::{DataHelper, EntityIter, System};

use super::LevelServices;

use crate::components::LevelComponents;

//use game::Interaction;

pub struct InteractionSystem;

impl System for InteractionSystem {
    type Components = LevelComponents;
    type Services = LevelServices;
}

impl EntityProcess for InteractionSystem {
    fn process(
        &mut self,
        _entities: EntityIter<'_, LevelComponents>,
        _data: &mut DataHelper<LevelComponents, LevelServices>,
    ) {
    }
}
