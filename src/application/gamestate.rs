use std::thread;
use std::collections::HashMap;
use std::path::Path;

use glium::{self};
use glium::glutin::{self, ElementState, VirtualKeyCode};

use ecs::{World, BuildData};
use ecs::system::{InteractSystem};

use util::{State, TextureStore};
use application::{AppTransition, InputIntent, InputState, InputManager};

use systems::{LevelSystems, RenderSystem, WorldViewport};
use components::{LevelComponents, Position, SpriteInfo, Camera, KeyboardInput, Intents};

use hprof;

use clock_ticks;

use std::time::Duration;

use nc::bounding_volume::AABB2;
use na::Pnt2;

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
        let render_system = RenderSystem::new(self.display.clone());

        world.services.texture_store = TextureStore::new(self.display.clone());

        world.systems.render_system.init(InteractSystem::new(
            render_system,
            aspect!(<LevelComponents> all: [camera]),
            aspect!(<LevelComponents> all: [position, sprite_info])
                ));

        let tex_info = world.services.texture_store.get_texture_info(Path::new("assets/textures/tilesets/cave/tile1.png"));

        let _ = world.create_entity(
            |entity: BuildData<LevelComponents>, data: &mut LevelComponents| {
                data.position.add(&entity, Position { x: ( width - 32 ) as f32, y: ( height - 32 ) as f32 });
                data.sprite_info.add(&entity, SpriteInfo {
                    width: 32.0,
                    height: 32.0,
                    texture_info: tex_info,
                });
                data.camera.add(&entity, Camera::new(
                    WorldViewport::new((width / 1) as f32, ( height / 1 )as f32),
                    AABB2::new(Pnt2::new(-1.0, -1.0), Pnt2::new(1.0, 1.0)),
                    true
                    ));
                data.intents.add(&entity, Intents::new());
                let mut inputs = HashMap::new();
                inputs.insert((VirtualKeyCode::O, InputState::PressedThisFrame), InputIntent::PrintDebugMessage);

                data.keyboard_input.add(&entity, KeyboardInput{
                    input_context: inputs,
                });
            }
            );

        process!(world, camera_system);

        let mut profiler_ticks = 0;

        let mut previous_time = clock_ticks::precise_time_ns();
        let mut lag_behind_simulation = 0u64;

        // change these
        const MS_PER_UPDATE: u64 = 10;
        #[allow(dead_code)]
        const FPS: u64 = 60;

        // leave these
        const MS_TO_NS: u64 = 1000000;
        const NS_PER_UPDATE: u64 = MS_PER_UPDATE * MS_TO_NS;
        #[allow(dead_code)]
        const INV_FPS_NS: u64 = 1000000000 / FPS; // 1s / FPS

        // change this to min(NS_PER_UPDATE, INV_FPS_NS)
        const MAX_SLEEP: u64 = NS_PER_UPDATE;

        let mut input_manager = InputManager::new();

        loop {
            hprof::start_frame();

            let current_time = clock_ticks::precise_time_ns();
            let elapsed = current_time - previous_time;
            previous_time = current_time;
            lag_behind_simulation += elapsed;

            {
                let _ = hprof::enter("window-events");

                for event in self.display.poll_events() {
                    match event {
                        glutin::Event::Closed |
                        glutin::Event::KeyboardInput(ElementState::Released, _, Some(VirtualKeyCode::Escape))
                            => return AppTransition::Shutdown,
                        glutin::Event::KeyboardInput(ElementState::Released, _, Some(VirtualKeyCode::P))
                            => profiler_ticks += 3,
                        glutin::Event::KeyboardInput(ElementState::Pressed, _, Some(vkc))
                            => {
                                input_manager.handle_event(ElementState::Pressed, vkc);
                                println!("pressed: {:?}", vkc);
                            },
                        glutin::Event::KeyboardInput(ElementState::Released, _, Some(vkc))
                            => {
                                input_manager.handle_event(ElementState::Released, vkc);
                                println!("released: {:?}", vkc);
                            },
                        glutin::Event::ReceivedCharacter(c)
                            => println!("char: {:?}", c),
                        glutin::Event::Resized(width, height) => {
                            world.systems.camera_system.resized = Some((width, height));
                            process!(world, camera_system);
                        },
                        _ => ()
                    }
                }
            }

            input_manager.dispatch(&mut world.systems.keyboard_system.inner);
            input_manager.end_frame();

            while lag_behind_simulation >= NS_PER_UPDATE {
                let _ = hprof::enter("world-update");
                world.update();
                lag_behind_simulation -= NS_PER_UPDATE;
            }

            process!(world, intent_system);
            process!(world, render_system);

            hprof::end_frame();

            let diff = clock_ticks::precise_time_ns() - previous_time;
            if diff < MAX_SLEEP {
                thread::sleep(Duration::new(0, (MAX_SLEEP - diff) as u32));
            }

            if profiler_ticks > 0 {
                hprof::profiler().print_timing();
                profiler_ticks -= 1;
            }
        }
    }
}
