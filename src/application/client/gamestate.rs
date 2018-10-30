use glium::{self, glutin};

use crate::net;
use crate::application::client::ClientTransition;
use crate::util::State;

pub struct GameState {
    display: glium::Display,
    events_loop: glutin::EventsLoop,
}

impl GameState {
    pub fn new(display: glium::Display, events_loop: glutin::EventsLoop) -> GameState {
        GameState {
            display,
            events_loop,
        }
    }
}

impl State<ClientTransition> for GameState {
    fn run(mut self: Box<Self>) -> ClientTransition {
        panic!()
    }
}
