pub use self::render_system::{ RenderSystem, WorldViewport };
pub use self::camera_system::{ CameraSystem };
pub use self::keyboard_system::KeyboardSystem;
pub use self::intent_system::IntentSystem;
pub use self::velocity_system::VelocitySystem;
pub use self::gravity_system::GravitySystem;
pub use self::collision_system::{CollisionSystem};
pub use self::movement_system::MovementSystem;

use ecs::ServiceManager;
use ecs::system::{ LazySystem, EntitySystem, InteractSystem };

use components::LevelComponents;

use util::TextureStore;

mod render_system;
mod camera_system;
mod keyboard_system;
mod intent_system;
mod velocity_system;
mod gravity_system;
mod collision_system;
mod movement_system;

#[derive(Default)]
pub struct LevelServices {
    pub texture_store: TextureStore // = TextureStore::new_invalid()
        ,
    pub delta_time_s: f32 // = 0.0
        ,
}

impl ServiceManager for LevelServices {}

systems! {
    struct LevelSystems<LevelComponents, LevelServices> {
        keyboard_system: EntitySystem<KeyboardSystem> = EntitySystem::new(
            KeyboardSystem::new(),
            aspect!(<LevelComponents> all: [keyboard_input])),
        gravity_system: EntitySystem<GravitySystem> = EntitySystem::new(
            GravitySystem { g: 99.0 },
            aspect!(<LevelComponents> all: [gravity, velocity]),
            ),
        movement_system: EntitySystem<MovementSystem> = EntitySystem::new(
            MovementSystem,
            aspect!(<LevelComponents> all: [velocity, movement, intents]),
            ),
        velocity_system: EntitySystem<VelocitySystem> = EntitySystem::new(
            VelocitySystem,
            aspect!(<LevelComponents> all: [velocity]),
            ),
        collision_system: InteractSystem<CollisionSystem> = InteractSystem::new(
            CollisionSystem::new(),
            aspect!(<LevelComponents> all: [position, velocity, collision]),
            aspect!(<LevelComponents> all: [position, collision]),
            ),
        camera_system: EntitySystem<CameraSystem> = EntitySystem::new(
            CameraSystem::new(),
            aspect!(<LevelComponents> all: [camera])),
        render_system: LazySystem<InteractSystem<RenderSystem>> = LazySystem::new(),
        intent_system: EntitySystem<IntentSystem> = EntitySystem::new(
            IntentSystem,
            aspect!(<LevelComponents> all: [intents])),
    }
}

