use ecs::system::EntityProcess;
use ecs::{DataHelper, EntityIter, System};

use super::LevelServices;

use crate::components::LevelComponents;

use crate::application::InputIntent;

pub struct IntentSystem;

impl System for IntentSystem {
    type Components = LevelComponents;
    type Services = LevelServices;
}

impl EntityProcess for IntentSystem {
    fn process(
        &mut self,
        entities: EntityIter<'_, LevelComponents>,
        data: &mut DataHelper<LevelComponents, LevelServices>,
    ) {
        for e in entities {
            if data.intents[e].contains(&InputIntent::PrintDebugMessage) {
                println!("debug message!");
            }

            data.intents[e].clear();
        }
    }
}
