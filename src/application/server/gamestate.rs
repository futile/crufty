use std::collections::HashMap;
use std::time::Duration;
use std::path::Path;
use std::thread;

use glium;
use glium::glutin::{self, ElementState, VirtualKeyCode};

use ecs::system::InteractSystem;
use ecs::{BuildData /* , ModifyData */, World};

use crate::application::{
    client::ClientTransition, server::ServerTransition, InputIntent, InputManager, InputState,
};
use crate::game::{Interaction, ResourceStore};
use crate::net;
use crate::util::State;

use crate::components::{
    Camera, CollisionShape, CollisionType, Facing, Gravity, Intents, InteractionPossibility,
    Interactor, Jump, KeyboardInput, LevelComponents, Movement, Position, Sprite, SpriteInfo,
    SpriteLayer, SpriteSheetAnimation, Velocity,
};
use crate::systems::{LevelSystems, RenderSystem, WorldViewport};

use hprof;

use clock_ticks;


use crate::na::{Point2, Vector2};
use crate::nc::bounding_volume::AABB;
use crate::nc::shape::Cuboid;

pub struct GameState {
    display: glium::Display,
    events_loop: glutin::EventsLoop,
    host: net::Host,
}

impl GameState {
    pub fn new(
        display: glium::Display,
        events_loop: glutin::EventsLoop,
        host: net::Host,
    ) -> GameState {
        GameState {
            display,
            events_loop,
            host,
        }
    }
}

impl State<ServerTransition> for GameState {
    fn run(mut self: Box<Self>) -> ServerTransition {
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

        let tex_info = world
            .services
            .resource_store
            .load_texture(Path::new("assets/textures/tilesets/cave/tile1.png"));

        let player_tex_info = world.services.resource_store.load_texture(Path::new(
            "assets/textures/sprites/player/stand/p_stand.png",
        ));
        // .load_texture(Path::new("assets/textures/sprites/player/jump/p_jump.png"));

        let player_stand_animation = world
            .services
            .resource_store
            .get_sprite_sheet(ss_handle)
            .get("stand")
            .unwrap()
            .clone();

        let _ = world.create_entity(
            |entity: BuildData<'_, LevelComponents>, data: &mut LevelComponents| {
                data.position.add(&entity, Position { x: 0.0, y: 0.0 });
                data.camera.add(
                    &entity,
                    Camera::new(
                        WorldViewport::new((width / 1) as f32, (height / 1) as f32),
                        AABB::new(Point2::new(-1.0, -1.0), Point2::new(1.0, 1.0)),
                        true,
                    ),
                );
            },
        );

        let _player = {
            let position = Position {
                x: 8.0 * 32.0 + 0.0 * 10.0,
                // y: 500.0,
                y: 0.0,
            };
            let velocity = Velocity {
                vx: 00.0,
                vy: 00.0,
                last_pos: position,
            };
            let collision_shape = CollisionShape::new_dual(
                Cuboid::new(Vector2::new(16.0, 5.0)),
                Vector2::new(16.0, 16.0),
                Cuboid::new(Vector2::new(5.0, 16.0)),
                Vector2::new(16.0, 16.0),
                CollisionType::Solid,
            );
            let movement = Movement::new(Vector2::new(75.0, 0.0), Vector2::new(150.0, 0.0));
            let facing = Facing::Right;
            let jump = Jump::new();
            let gravity = Gravity::new();
            let sprite = Sprite {
                info: SpriteInfo {
                    width: 32.0,
                    height: 32.0,
                    texture_info: player_tex_info,
                },
                sprite_layer: SpriteLayer::Foreground,
            };
            let ss_anim = SpriteSheetAnimation {
                sheet_handle: ss_handle,
                animation: player_stand_animation.clone(),
                current_frame: 0,
                frame_time_remaining: 0.1,
            };
            let intents = Intents::new();
            let interactor = Interactor;
            let kb_input = KeyboardInput {
                input_context: {
                    let mut inputs = HashMap::new();
                    inputs.insert(
                        (VirtualKeyCode::O, InputState::PressedThisFrame),
                        InputIntent::PrintDebugMessage,
                    );
                    inputs.insert(
                        (VirtualKeyCode::Left, InputState::Pressed),
                        InputIntent::MoveLeft,
                    );
                    inputs.insert(
                        (VirtualKeyCode::Right, InputState::Pressed),
                        InputIntent::MoveRight,
                    );
                    inputs.insert(
                        (VirtualKeyCode::Space, InputState::Pressed),
                        InputIntent::Jump,
                    );
                    inputs.insert(
                        (VirtualKeyCode::E, InputState::PressedThisFrame),
                        InputIntent::Interact,
                    );
                    inputs
                },
            };

            let _player = world.create_entity(
                |entity: BuildData<'_, LevelComponents>, data: &mut LevelComponents| {
                    data.position.add(&entity, position);
                    data.velocity.add(&entity, velocity);
                    data.collision_shape.add(&entity, collision_shape.clone());
                    data.movement.add(&entity, movement.clone());
                    data.facing.add(&entity, facing);
                    data.jump.add(&entity, jump);
                    data.gravity.add(&entity, gravity);
                    data.sprite.add(&entity, sprite.clone());
                    data.sprite_sheet_animation.add(&entity, ss_anim.clone());
                    data.intents.add(&entity, intents.clone());
                    data.interactor.add(&entity, interactor);
                    data.keyboard_input.add(&entity, kb_input.clone());
                },
            );

            world
                .services
                .changed_flags
                .position
                .insert(_player, position);
            world
                .services
                .changed_flags
                .velocity
                .insert(_player, velocity);
            world
                .services
                .changed_flags
                .collision_shape
                .insert(_player, collision_shape);
            world
                .services
                .changed_flags
                .movement
                .insert(_player, movement);
            world.services.changed_flags.facing.insert(_player, facing);
            world.services.changed_flags.jump.insert(_player, jump);
            world
                .services
                .changed_flags
                .gravity
                .insert(_player, gravity);
            world.services.changed_flags.sprite.insert(_player, sprite);
            world
                .services
                .changed_flags
                .sprite_sheet_animation
                .insert(_player, ss_anim);
            world
                .services
                .changed_flags
                .intents
                .insert(_player, intents);
            world
                .services
                .changed_flags
                .interactor
                .insert(_player, interactor);
            world
                .services
                .changed_flags
                .keyboard_input
                .insert(_player, kb_input);

            _player
        };

        for x in 0..12 {
            let _ = {
                let position = Position {
                    x: (x as f32) * 32.0,
                    y: 0.0,
                };
                let collision_shape = CollisionShape::new_single(
                    Cuboid::new(Vector2::new(16.0, 16.0)),
                    Vector2::new(16.0, 16.0),
                    CollisionType::Solid,
                );
                let sprite = Sprite {
                    info: SpriteInfo {
                        width: 32.0,
                        height: 32.0,
                        texture_info: tex_info,
                    },
                    sprite_layer: SpriteLayer::Background,
                };

                let _e = world.create_entity(
                    |entity: BuildData<'_, LevelComponents>, data: &mut LevelComponents| {
                        data.position.add(&entity, position);
                        data.collision_shape.add(&entity, collision_shape.clone());
                        data.sprite.add(&entity, sprite.clone());
                    },
                );

                world.services.changed_flags.position.insert(_e, position);

                world
                    .services
                    .changed_flags
                    .collision_shape
                    .insert(_e, collision_shape);
                world.services.changed_flags.sprite.insert(_e, sprite);

                _e
            };
        }

        for x in 1..12 {
            let _ = {
                let position = Position {
                    x: (x as f32) * 32.0,
                    y: 96.0,
                };
                let collision_shape = CollisionShape::new_single(
                    Cuboid::new(Vector2::new(16.0, 16.0)),
                    Vector2::new(16.0, 16.0),
                    CollisionType::Solid,
                );
                let sprite = Sprite {
                    info: SpriteInfo {
                        width: 32.0,
                        height: 32.0,
                        texture_info: tex_info,
                    },
                    sprite_layer: SpriteLayer::Background,
                };

                let _e = world.create_entity(
                    |entity: BuildData<'_, LevelComponents>, data: &mut LevelComponents| {
                        data.position.add(&entity, position);
                        data.collision_shape.add(&entity, collision_shape.clone());
                        data.sprite.add(&entity, sprite.clone());
                    },
                );

                world.services.changed_flags.position.insert(_e, position);

                world
                    .services
                    .changed_flags
                    .collision_shape
                    .insert(_e, collision_shape);
                world.services.changed_flags.sprite.insert(_e, sprite);

                _e
            };
        }

        let _warp_block = {
            let position = Position {
                x: 10. * 32.0,
                y: 32.0,
            };
            let collision_shape = CollisionShape::new_single(
                Cuboid::new(Vector2::new(16.0, 16.0)),
                Vector2::new(16.0, 16.0),
                CollisionType::Trigger,
            );
            let interaction_possibility = InteractionPossibility {
                interaction: Interaction::WarpInRoom { x: 0.0, y: 500.0 },
            };
            let sprite = Sprite {
                info: SpriteInfo {
                    width: 32.0,
                    height: 32.0,
                    texture_info: player_tex_info,
                },
                sprite_layer: SpriteLayer::Background,
            };

            let _e = world.create_entity(
                |entity: BuildData<'_, LevelComponents>, data: &mut LevelComponents| {
                    data.position.add(&entity, position);
                    data.collision_shape.add(&entity, collision_shape.clone());
                    data.interaction_possibility
                        .add(&entity, interaction_possibility);
                    data.sprite.add(&entity, sprite.clone());
                },
            );

            world.services.changed_flags.position.insert(_e, position);
            world
                .services
                .changed_flags
                .collision_shape
                .insert(_e, collision_shape);
            world
                .services
                .changed_flags
                .interaction_possibility
                .insert(_e, interaction_possibility);
            world.services.changed_flags.sprite.insert(_e, sprite);

            _e
        };

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
                    return ServerTransition::Shutdown;
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

            self.host.maintain(&mut world);

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

impl State<ClientTransition> for GameState {
    fn run(self: Box<Self>) -> ClientTransition {
        ClientTransition::Shutdown
    }
}
