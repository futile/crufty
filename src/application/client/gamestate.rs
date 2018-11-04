use std::path::Path;
use std::time::Duration;
use std::thread;

use glium::{self};
use glium::glutin::{self, ElementState, VirtualKeyCode};
use ecs::{World, system::InteractSystem};

use crate::application::{client::ClientTransition, InputManager};
use crate::net;
use crate::util::State;
use crate::systems::{LevelSystems, RenderSystem};
use crate::game::ResourceStore;
use crate::components::LevelComponents;

pub struct GameState {
    display: glium::Display,
    events_loop: glutin::EventsLoop,
    client: net::Client,
}

impl GameState {
    pub fn new(
        display: glium::Display,
        events_loop: glutin::EventsLoop,
        client: net::Client,
    ) -> GameState {
        GameState {
            display,
            events_loop,
            client,
        }
    }
}

impl State<ClientTransition> for GameState {
    fn run(mut self: Box<Self>) -> ClientTransition {
        let mut world = World::<LevelSystems>::new();

        let (width, height) = self.display.get_framebuffer_dimensions();
        let render_system = RenderSystem::new(self.display.clone());

        world.services.resource_store = ResourceStore::new(self.display.clone());

        let ss_handle = world
            .services
            .resource_store
            .load_sprite_sheet(Path::new("assets/textures/sprites/player/animations.toml"));

        world.systems.render_system.init(InteractSystem::new(
            render_system,
            aspect!(<LevelComponents> all: [camera]),
            aspect!(<LevelComponents> all: [position]),
        ));

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

        world.services.delta_time_s = (MS_PER_UPDATE as f32) / 1000.0;

        let mut input_manager = InputManager::new();

        loop {
            hprof::start_frame();

            let current_time = clock_ticks::precise_time_ns();
            let elapsed = current_time - previous_time;
            previous_time = current_time;
            lag_behind_simulation += elapsed;

            {
                let _ = hprof::enter("window-events");

                let mut shutdown = false;

                self.events_loop.poll_events(|event| {
                    use self::glutin::{dpi::LogicalSize, Event, KeyboardInput, WindowEvent};
                    let event = match event {
                        Event::WindowEvent {
                            window_id: _,
                            event,
                        } => event,
                        _ => return,
                    };

                    match event {
                        WindowEvent::CloseRequested => {
                            shutdown = true;
                        }
                        WindowEvent::KeyboardInput {
                            device_id: _,
                            input:
                                KeyboardInput {
                                    scancode: _,
                                    state: key_state,
                                    modifiers: _,
                                    virtual_keycode: Some(vkc),
                                },
                        } => match (key_state, vkc) {
                            (ElementState::Released, VirtualKeyCode::P) => profiler_ticks += 3,
                            (ElementState::Released, VirtualKeyCode::D) => {
                                world
                                    .systems
                                    .render_system
                                    .inner
                                    .as_mut()
                                    .unwrap()
                                    .toggle_physics_debug_render();
                            }
                            (ElementState::Released, VirtualKeyCode::Escape) => shutdown = true,
                            _ => input_manager.handle_event(key_state, vkc),
                        },
                        WindowEvent::Resized(LogicalSize { width, height }) => {
                            world.systems.camera_system.resized =
                                Some((width as u32, height as u32));
                            process!(world, camera_system);
                        }
                        _ => (),
                    }
                });

                if shutdown {
                    return ClientTransition::Shutdown;
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

            self.client.maintain();

            world.services.changed_flags.clear();

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
