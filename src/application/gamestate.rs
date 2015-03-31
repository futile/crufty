use std::thread;
use std::time::Duration;

use glium::{self, Surface};
use glutin::{self, ElementState, VirtualKeyCode};

use ecs::{World, BuildData};

use util::{State};
use application::AppTransition;

use systems::RenderSystem;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

components! {
    MyComponents {
        #[hot] position: Position
    }
}

systems! {
    MySystems<MyComponents, ()>;
}

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
            let mut world = World::<MySystems>::new();

            let entity = world.create_entity(
                |entity: BuildData<MyComponents>, data: &mut MyComponents| {
                    data.position.add(&entity, Position { x: 0.0, y: 0.0 });
                }
                );

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
