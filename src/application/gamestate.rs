use glium::{self, Surface};
use glutin::{self, ElementState, VirtualKeyCode};

use std::thread;
use std::time::Duration;

use util::{State};
use application::AppTransition;

pub struct GameState {
    display: glium::Display,
}

impl GameState {
    pub fn new(display: glium::Display) -> GameState {
        GameState{
            display: display,
        }
    }
}

impl State<AppTransition> for GameState {
    fn run(self: Box<Self>) -> AppTransition {
        loop {
            for event in self.display.poll_events() {
                let mut target = self.display.draw();
                target.clear_color(0.0, 0.0, 0.0, 0.0);
                target.finish();

                match event {
                    glutin::Event::Closed |
                    glutin::Event::KeyboardInput(ElementState::Released, _, Some(VirtualKeyCode::Escape))
                        => return AppTransition::Shutdown,
                    _ => ()
                }

                thread::sleep(Duration::milliseconds(17));
            }
        }
    }
}
