use std::thread;

use glium::{self};
use glutin::{self, ElementState, VirtualKeyCode};

use ecs::{World, BuildData};

use util::{State};
use application::AppTransition;

use systems::{LevelSystems, RenderSystem};
use components::{LevelComponents, Position};

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
            let mut world = World::<LevelSystems>::new();

            world.systems.render_system.init(RenderSystem{display: self.display.clone()});

            let _ = world.create_entity(
                |entity: BuildData<LevelComponents>, data: &mut LevelComponents| {
                    data.position.add(&entity, Position { x: 0.0, y: 0.0 });
                }
                );

            world.update();

            for event in self.display.poll_events() {
                match event {
                    glutin::Event::Closed |
                    glutin::Event::KeyboardInput(ElementState::Released, _, Some(VirtualKeyCode::Escape))
                        => return AppTransition::Shutdown,
                    _ => ()
                }

                thread::sleep_ms(17);
            }
        }
    }
}
