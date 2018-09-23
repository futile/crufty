use std::thread;
use std::collections::HashMap;
use std::path::Path;

use glium;
use glium::glutin::{self, ElementState, VirtualKeyCode};

use ecs::{World, BuildData /* , ModifyData */};
use ecs::system::InteractSystem;

use crate::util::State;
use crate::application::{AppTransition, InputIntent, InputState, InputManager};
use crate::game::ResourceStore;

use crate::systems::{LevelSystems, RenderSystem, WorldViewport};
use crate::components::{LevelComponents, Movement, Jump, Position, Collision, CollisionType, SpriteInfo,
                 SpriteSheetAnimation, Gravity, Camera, KeyboardInput, Intents, Velocity, Facing};

use hprof;

use clock_ticks;

use std::time::Duration;

use crate::nc::shape::Cuboid;
use crate::nc::bounding_volume::AABB;
use crate::na::{Point2, Vector2};

pub struct GameState {
    display: glium::Display,
    events_loop: glutin::EventsLoop,
}

impl GameState {
    pub fn new(display: glium::Display, events_loop: glutin::EventsLoop) -> GameState {
        GameState { display, events_loop }
    }
}

impl State<AppTransition> for GameState {
    fn run(mut self: Box<Self>) -> AppTransition {
        let mut world = World::<LevelSystems>::new();

        let (width, height) = self.display.get_framebuffer_dimensions();
        let render_system = RenderSystem::new(self.display.clone());

        world.services.resource_store = ResourceStore::new(self.display.clone());

        let ss_handle = world.services
            .resource_store
            .load_sprite_sheet(Path::new("assets/textures/sprites/player/animations.toml"));

        world.systems
            .render_system
            .init(InteractSystem::new(render_system,
                                      aspect!(<LevelComponents> all: [camera]),
                                      aspect!(<LevelComponents> all: [position, sprite_info])));

        let tex_info = world.services
            .resource_store
            .load_texture(Path::new("assets/textures/tilesets/cave/tile1.png"));

        let player_tex_info = world.services
                                   .resource_store
                                   .load_texture(Path::new("assets/textures/sprites/player/stand/p_stand.png"));
                                    // .load_texture(Path::new("assets/textures/sprites/player/jump/p_jump.png"));

        let player_stand_animation = world.services
            .resource_store
            .get_sprite_sheet(ss_handle)
            .get("stand")
            .unwrap()
            .clone();

        let _ =
            world.create_entity(|entity: BuildData<'_, LevelComponents>,
                                 data: &mut LevelComponents| {
                data.position.add(&entity, Position { x: 0.0, y: 0.0 });
                data.camera.add(&entity,
                                Camera::new(WorldViewport::new((width / 1) as f32,
                                                               (height / 1) as f32),
                                            AABB::new(Point2::new(-1.0, -1.0),
                                                       Point2::new(1.0, 1.0)),
                                            true));
            });

        let _player =
            world.create_entity(|entity: BuildData<'_, LevelComponents>,
                                 data: &mut LevelComponents| {
                let pos = Position {
                    x: 0.0 * 32.0 + 0.0 * 10.0,
                    y: 500.0,
                };
                data.position.add(&entity, pos);
                data.velocity.add(&entity,
                                  Velocity {
                                      vx: 00.0,
                                      vy: 00.0,
                                      last_pos: pos,
                                  });
                data.collision.add(&entity,
                                   Collision::new_dual(Cuboid::new(Vector2::new(16.0, 5.0)),
                                                       Vector2::new(16.0, 16.0),
                                                       Cuboid::new(Vector2::new(5.0, 16.0)),
                                                       Vector2::new(16.0, 16.0),
                                                       CollisionType::Solid));
                data.movement.add(&entity,
                                  Movement::new(Vector2::new(75.0, 0.0), Vector2::new(150.0, 0.0)));
                data.facing.add(&entity, Facing::Right);
                data.jump.add(&entity, Jump::new());
                data.gravity.add(&entity, Gravity::new());
                data.sprite_info.add(&entity,
                                     SpriteInfo {
                                         width: 32.0,
                                         height: 32.0,
                                         texture_info: player_tex_info,
                                     });
                data.sprite_sheet_animation.add(&entity,
                                                SpriteSheetAnimation {
                                                    sheet_handle: ss_handle,
                                                    animation: player_stand_animation.clone(),
                                                    current_frame: 0,
                                                    frame_time_remaining: 0.1,
                                                });
                data.intents.add(&entity, Intents::new());

                data.keyboard_input.add(&entity,
                                        KeyboardInput {
                                            input_context: {
                                                let mut inputs = HashMap::new();
                                                inputs.insert((VirtualKeyCode::O,
                                                               InputState::PressedThisFrame),
                                                              InputIntent::PrintDebugMessage);
                                                inputs.insert((VirtualKeyCode::Left,
                                                               InputState::Pressed),
                                                              InputIntent::MoveLeft);
                                                inputs.insert((VirtualKeyCode::Right,
                                                               InputState::Pressed),
                                                              InputIntent::MoveRight);
                                                inputs.insert((VirtualKeyCode::Space,
                                                               InputState::Pressed),
                                                              InputIntent::Jump);
                                                inputs
                                            },
                                        });
            });

        for x in 0..12 {
            let _ =
                world.create_entity(|entity: BuildData<'_, LevelComponents>,
                                     data: &mut LevelComponents| {
                    data.position.add(&entity,
                                      Position {
                                          x: (x as f32) * 32.0,
                                          y: 0.0,
                                      });
                    data.collision.add(&entity,
                                       Collision::new_single(Cuboid::new(Vector2::new(16.0,
                                                                                       16.0)),
                                                             Vector2::new(16.0, 16.0),
                                                             CollisionType::Solid));
                    data.sprite_info.add(&entity,
                                         SpriteInfo {
                                             width: 32.0,
                                             height: 32.0,
                                             texture_info: tex_info,
                                         });
                });
        }

        for x in 1..12 {
            let _ =
                world.create_entity(|entity: BuildData<'_, LevelComponents>,
                                     data: &mut LevelComponents| {
                    data.position.add(&entity,
                                      Position {
                                          x: (x as f32) * 32.0,
                                          y: 96.0,
                                      });
                    data.collision.add(&entity,
                                       Collision::new_single(Cuboid::new(Vector2::new(16.0,
                                                                                       16.0)),
                                                             Vector2::new(16.0, 16.0),
                                                             CollisionType::Trigger));
                    data.sprite_info.add(&entity,
                                         SpriteInfo {
                                             width: 32.0,
                                             height: 32.0,
                                             texture_info: tex_info,
                                         });
                });
        }

        let _ =
            world.create_entity(|entity: BuildData<'_, LevelComponents>,
                                 data: &mut LevelComponents| {
                data.position.add(&entity,
                                  Position {
                                      x: 352.0,
                                      y: 32.0,
                                  });
                data.collision.add(&entity,
                                   Collision::new_single(Cuboid::new(Vector2::new(16.0, 16.0)),
                                                         Vector2::new(16.0, 16.0),
                                                         CollisionType::Solid));
                data.sprite_info.add(&entity,
                                     SpriteInfo {
                                         width: 32.0,
                                         height: 32.0,
                                         texture_info: tex_info,
                                     });
            });

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
                    use self::glutin::{Event, WindowEvent, KeyboardInput, dpi::LogicalSize};
                    let event = match event {
                        Event::WindowEvent{window_id: _, event} => event,
                        _ => return,
                    };

                    match event {
                        WindowEvent::CloseRequested => {
                            shutdown = true;
                        }
                        WindowEvent::KeyboardInput{device_id: _, input: KeyboardInput{scancode: _,
                                                                                      state: key_state,
                                                                                      modifiers: _,
                                                                                      virtual_keycode: Some(vkc)}} => {
                            match (key_state, vkc) {
                                (ElementState::Released, VirtualKeyCode::P) => profiler_ticks += 3,
                                (ElementState::Released, VirtualKeyCode::Escape) => shutdown = true,
                                _ => input_manager.handle_event(key_state, vkc),
                            }
                        }
                        WindowEvent::Resized(LogicalSize {width, height}) => {
                            world.systems.camera_system.resized = Some((width as u32, height as u32));
                            process!(world, camera_system);
                        }
                        _ => (),
                    }
                });

                if shutdown {
                    return AppTransition::Shutdown
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
