use std::thread;

use glium::{self};
use glutin::{self, ElementState, VirtualKeyCode};

use ecs::{World, BuildData};
use ecs::system::EntitySystem;

use util::{State};
use application::AppTransition;

use systems::{LevelSystems, RenderSystem, WorldViewport};
use components::{LevelComponents, Position, SpriteInfo};

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

        let (width, height) = self.display.get_framebuffer_dimensions();
        let mut render_system = RenderSystem::new(self.display.clone());
        render_system.world_viewport = WorldViewport::new(0.0, 0.0, width as f32, height as f32);

        println!("width: {}, height: {}", width, height);

        world.systems.render_system.init(EntitySystem::new(
            render_system,
            aspect!(<LevelComponents> all: [position, sprite_info])
                ));


        let _ = world.create_entity(
            |entity: BuildData<LevelComponents>, data: &mut LevelComponents| {
                data.position.add(&entity, Position { x: 0.0, y: 0.0 });
                data.sprite_info.add(&entity, SpriteInfo { width: 32.0, height: 32.0 });
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
