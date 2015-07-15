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

    pub fn is_pressed(&self, vkc: &VirtualKeyCode) -> bool {
       self.keys.contains(vkc)
    }

    pub fn pressed_keys<'a>(&'a self) -> ::std::collections::hash_set::Iter<'a, VirtualKeyCode> {
        self.keys.iter()
    }
}

pub trait KeyHandler {
    // returns whether the key was consumed
    fn handle_key(&mut self, state: ElementState, new_this_frame: bool, key: VirtualKeyCode) -> bool;
}

pub struct InputManager {
    keyboard_state: KeyboardState,

    new_this_frame: HashSet<VirtualKeyCode>,
    consumed: HashSet<VirtualKeyCode>,
}

impl InputManager {
    pub fn new() -> InputManager {
        InputManager {
            keyboard_state: KeyboardState::new(),

            new_this_frame: HashSet::new(),
            consumed: HashSet::new(),
        }
    }

    pub fn handle_event(&mut self, state: ElementState, vkc: VirtualKeyCode) {
        self.keyboard_state.handle_event(state, vkc);
        self.new_this_frame.insert(vkc);
    }

    pub fn dispatch<T: KeyHandler>(&mut self, key_handler: &mut T) {
        for vkc in &self.new_this_frame {
            if(self.consumed.contains(&vkc)) {
                continue;
            }

            let state = if self.keyboard_state.is_pressed(vkc) { ElementState::Pressed } else { ElementState::Released };

            if key_handler.handle_key(state, true, *vkc) {
                self.consumed.insert(*vkc);
            }
        }

        for vkc in self.keyboard_state.pressed_keys() {
            if self.consumed.contains(&vkc) {
                continue;
            }

            if key_handler.handle_key(ElementState::Pressed, false, *vkc) {
                self.consumed.insert(*vkc);
            }
        }
    }

    pub fn end_frame(&mut self) {
        self.new_this_frame.clear();
        self.consumed.clear();
    }
}
