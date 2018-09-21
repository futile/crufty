use ecs::{System, DataHelper, EntityIter};
use ecs::system::EntityProcess;

use glium::glutin::VirtualKeyCode;

use std::collections::HashSet;

use super::LevelServices;

use crate::application::{KeyHandler, InputState};

use crate::components::LevelComponents;

pub struct KeyboardSystem {
    keys: HashSet<(VirtualKeyCode, InputState)>,
}

impl KeyboardSystem {
    pub fn new() -> KeyboardSystem {
        KeyboardSystem { keys: HashSet::new() }
    }
}

impl System for KeyboardSystem {
    type Components = LevelComponents;
    type Services = LevelServices;
}

impl KeyHandler for KeyboardSystem {
    fn handle_key(&mut self, state: InputState, key: VirtualKeyCode) -> bool {
        self.keys.insert((key, state));

        true
    }
}

impl EntityProcess for KeyboardSystem {
    fn process(&mut self,
               entities: EntityIter<'_, LevelComponents>,
               data: &mut DataHelper<LevelComponents, LevelServices>) {
        for e in entities {
            for combination in &self.keys {
                if let Some(&intent) = data.keyboard_input[e].input_context.get(combination) {
                    // add intent to IntentComponent
                    if !data.intents.has(&e) {
                        println!("entity has no intents");

                        continue;
                    }

                    data.intents[e].insert(intent);
                }
            }
        }

        self.keys.clear();
    }
}
