use ecs::{System, DataHelper, EntityIter};
use ecs::system::EntityProcess;

use super::LevelServices;

use crate::components::LevelComponents;

//use game::Interaction;

pub struct InteractionSystem;

impl System for InteractionSystem {
    type Components = LevelComponents;
    type Services = LevelServices;
}

impl EntityProcess for InteractionSystem {
    fn process(&mut self,
               _entities: EntityIter<'_, LevelComponents>,
               _data: &mut DataHelper<LevelComponents, LevelServices>) {
    }
}
