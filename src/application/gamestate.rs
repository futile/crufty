use std::thread;

use glium::{self};
use glutin::{self, ElementState, VirtualKeyCode};

use ecs::{World, BuildData};
use ecs::system::EntitySystem;

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
        let mut world = World::<LevelSystems>::new();

        world.systems.render_system.init(EntitySystem::new(
            RenderSystem::new(self.display.clone()),
            aspect!(<LevelComponents> all: [position])
                ));

        let _ = world.create_entity(
            |entity: BuildData<LevelComponents>, data: &mut LevelComponents| {
                data.position.add(&entity, Position { x: 0.0, y: 0.0 });
            }
            );

        loop {
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

            process!(world, render_system);
        }
    }
}
