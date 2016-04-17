pub use self::render_system::{RenderSystem, WorldViewport};
pub use self::camera_system::CameraSystem;
pub use self::keyboard_system::KeyboardSystem;
pub use self::intent_system::IntentSystem;
pub use self::velocity_system::VelocitySystem;
pub use self::gravity_system::GravitySystem;
pub use self::collision_system::CollisionSystem;
pub use self::movement_system::MovementSystem;
pub use self::jump_system::JumpSystem;
pub use self::ssanimation_system::SpriteSheetAnimationSystem;

use ecs::ServiceManager;
use ecs::system::{LazySystem, EntitySystem, InteractSystem};

use components::LevelComponents;

use util::{TextureStore, CollisionWorld};

mod render_system;
mod camera_system;
mod keyboard_system;
mod intent_system;
mod velocity_system;
mod gravity_system;
mod collision_system;
mod movement_system;
mod jump_system;
mod ssanimation_system;

pub struct LevelServices {
    pub texture_store: TextureStore,
    pub delta_time_s: f32,
    pub gravity: f32,
    pub collision_world: CollisionWorld,
}

impl Default for LevelServices {
    fn default() -> LevelServices {
        LevelServices {
            texture_store: TextureStore::new_invalid(),
            delta_time_s: 0.0,
            gravity: 150.0,
            collision_world: CollisionWorld::new(),
        }
    }
}

impl ServiceManager for LevelServices {}

systems! {
    struct LevelSystems<LevelComponents, LevelServices> {
        active: {
            keyboard_system: EntitySystem<KeyboardSystem> = EntitySystem::new(
                KeyboardSystem::new(),
                aspect!(<LevelComponents> all: [keyboard_input])),
            gravity_system: EntitySystem<GravitySystem> = EntitySystem::new(
                GravitySystem,
                aspect!(<LevelComponents> all: [gravity, velocity]),
            ),
            movement_system: EntitySystem<MovementSystem> = EntitySystem::new(
                MovementSystem,
                aspect!(<LevelComponents> all: [velocity, movement, intents]),
            ),
            jump_system: EntitySystem<JumpSystem> = EntitySystem::new(
                JumpSystem,
                aspect!(<LevelComponents> all: [velocity, jump, intents]),
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
            sprite_sheet_animation_system: EntitySystem<SpriteSheetAnimationSystem> = EntitySystem::new(
                SpriteSheetAnimationSystem::new(),
                aspect!(<LevelComponents> all: [sprite_sheet_animation, sprite_info]),
            ),
        },
        passive: {
            camera_system: EntitySystem<CameraSystem> = EntitySystem::new(
                CameraSystem::new(),
                aspect!(<LevelComponents> all: [camera])),
            render_system: LazySystem<InteractSystem<RenderSystem>> = LazySystem::new(),
            intent_system: EntitySystem<IntentSystem> = EntitySystem::new(
                IntentSystem,
                aspect!(<LevelComponents> all: [intents])),
        }
    }
}
