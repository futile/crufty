use glium::{self, glutin};

use crate::net;
use crate::application::client::ClientTransition;
use crate::util::State;

pub struct GameState {
    display: glium::Display,
    events_loop: glutin::EventsLoop,
    client: net::Client,
}

impl GameState {
    pub fn new(display: glium::Display, events_loop: glutin::EventsLoop, client: net::Client) -> GameState {
        GameState {
            display,
            events_loop,
            client,
        }
    }
}

impl State<ClientTransition> for GameState {
    fn run(mut self: Box<Self>) -> ClientTransition {
        panic!()
    }
}
