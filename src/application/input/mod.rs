use glium::glutin::{ElementState, VirtualKeyCode};

use std::collections::HashSet;
use std::collections::HashMap;

mod intents;

pub use self::intents::InputIntent;

struct KeyboardState {
    keys: HashSet<VirtualKeyCode>,
}

impl KeyboardState {
    pub fn new() -> KeyboardState {
        KeyboardState {
            keys: HashSet::new(),
        }
    }

    pub fn handle_event(&mut self, state: ElementState, vkc: VirtualKeyCode) {
        let _ = match state {
            ElementState::Pressed => self.keys.insert(vkc),
            ElementState::Released => self.keys.remove(&vkc),
        };
    }

    pub fn is_pressed(&self, vkc: VirtualKeyCode) -> bool {
        return self.keys.contains(&vkc);
    }
}

type InputContext = HashMap<VirtualKeyCode, InputIntent>;

pub struct MappedInputs {
    intent_to_key: HashMap<InputIntent, VirtualKeyCode>,
    consumed_keys: HashSet<VirtualKeyCode>,
}

impl MappedInputs {
    pub fn new() -> MappedInputs {
        MappedInputs {
            intent_to_key: HashMap::new(),
            consumed_keys: HashSet::new(),
        }
    }

    pub fn add_input(&mut self, intent: InputIntent, key: VirtualKeyCode) {
        self.intent_to_key.insert(intent, key);
    }

    pub fn has_intent(&self, intent: &InputIntent) -> bool {
        if let Some(vkc) = self.intent_to_key.get(intent) {
            return !self.consumed_keys.contains(vkc);
        }

        return false;
    }

    // returns `true` if the intent wasn't consumed before
    pub fn consume_intent(&mut self, intent: &InputIntent) -> bool {
        if let Some(vkc) = self.intent_to_key.get(intent) {
            return self.consumed_keys.insert(*vkc);
        }

        return false;
    }

    pub fn clear(&mut self) {
        self.intent_to_key.clear();
        self.consumed_keys.clear();
    }
}

pub struct InputManager {
    keyboard_state: KeyboardState,

    input_contexts: Vec<InputContext>,
}

impl InputManager {
    pub fn new() -> InputManager {
        InputManager {
            keyboard_state: KeyboardState::new(),

            input_contexts: Vec::new(),
        }
    }

    pub fn handle_event(&mut self, state: ElementState, vkc: VirtualKeyCode) {
        self.keyboard_state.handle_event(state, vkc);
    }

    fn is_pressed(&self, vkc: VirtualKeyCode) -> bool {
        self.keyboard_state.is_pressed(vkc)
    }
}
