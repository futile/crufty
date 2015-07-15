use glium::glutin::{ElementState, VirtualKeyCode};

use std::collections::HashSet;

pub struct KeyboardState {
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
