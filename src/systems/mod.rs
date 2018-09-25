pub use self::camera_system::CameraSystem;
pub use self::collision_system::CollisionSystem;
pub use self::gravity_system::GravitySystem;
pub use self::intent_system::IntentSystem;
pub use self::interaction_system::InteractionSystem;
pub use self::jump_system::JumpSystem;
pub use self::keyboard_system::KeyboardSystem;
pub use self::movement_system::MovementSystem;
pub use self::render_system::{RenderSystem, WorldViewport};
pub use self::sprite_sheet_animation_system::SpriteSheetAnimationSystem;
pub use self::velocity_system::VelocitySystem;

use ecs::system::{EntitySystem, InteractSystem, LazySystem};
use ecs::ServiceManager;

use crate::components::LevelComponents;

use crate::game::ResourceStore;
use crate::util::CollisionWorld;

mod camera_system;
mod collision_system;
mod gravity_system;
mod intent_system;
mod interaction_system;
mod jump_system;
mod keyboard_system;
mod movement_system;
mod render_system;
mod sprite_sheet_animation_system;
mod velocity_system;

pub struct LevelServices {
    pub resource_store: ResourceStore,
    pub delta_time_s: f32,
    pub gravity: f32,
    pub collision_world: CollisionWorld,
}

impl Default for LevelServices {
    fn default() -> LevelServices {
        LevelServices {
            resource_store: ResourceStore::default(),
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
            collision_system: EntitySystem<CollisionSystem> = EntitySystem::new(
                CollisionSystem::new(),
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
            interaction_system: EntitySystem<InteractionSystem> = EntitySystem::new(
                InteractionSystem,
                aspect!(<LevelComponents> all: [interaction_possibility]),
            ),
        }
    }
}
