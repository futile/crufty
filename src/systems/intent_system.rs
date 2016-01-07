use ecs::{System, DataHelper, EntityIter};
use ecs::system::EntityProcess;

use super::LevelServices;

use components::LevelComponents;

use application::InputIntent;

pub struct IntentSystem;

impl System for IntentSystem {
    type Components = LevelComponents;
    type Services = LevelServices;

    fn is_active(&self) -> bool {
        false
    }
}

impl EntityProcess for IntentSystem {
    fn process(&mut self,
               entities: EntityIter<LevelComponents>,
               data: &mut DataHelper<LevelComponents, LevelServices>) {
        for e in entities {
            if data.intents[e].contains(&InputIntent::PrintDebugMessage) {
                println!("debug message!");
            }

            data.intents[e].clear();
        }
    }
}
