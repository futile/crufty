use std::thread;

use glium::{self};
use glutin::{self, ElementState, VirtualKeyCode};

use ecs::{World, BuildData};
use ecs::system::{EntitySystem, InteractSystem};

use util::{State};
use application::AppTransition;

use systems::{LevelSystems, RenderSystem, WorldViewport};
use components::{LevelComponents, Position, SpriteInfo};

use hprof;

use clock_ticks;

use std::time::Duration;

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

        world.systems.render_system.init(InteractSystem::new(
            render_system,
            aspect!(<LevelComponents> all: [camera]),
            aspect!(<LevelComponents> all: [position, sprite_info])
                ));


        let _ = world.create_entity(
            |entity: BuildData<LevelComponents>, data: &mut LevelComponents| {
                data.position.add(&entity, Position { x: ( width - 32 ) as f32, y: ( height - 32 ) as f32 });
                data.sprite_info.add(&entity, SpriteInfo { width: 32.0, height: 32.0 });
            }
            );

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
                        glutin::Event::Resized(width, height) => {
                            let worldview = &mut world.systems.render_system.inner.as_mut().unwrap().inner.world_viewport;
                            worldview.width = width as f32; worldview.height = height as f32;
                        }
                        _ => ()
                    }
                }
            }

            while lag_behind_simulation >= NS_PER_UPDATE {
                let _ = hprof::enter("world-update");
                world.update();
                lag_behind_simulation -= NS_PER_UPDATE;
            }

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
