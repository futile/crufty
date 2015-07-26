pub use self::render_system::{ RenderSystem, WorldViewport };
pub use self::camera_system::{ CameraSystem };
pub use self::keyboard_system::KeyboardSystem;
pub use self::intent_system::IntentSystem;
pub use self::velocity_system::VelocitySystem;
pub use self::gravity_system::GravitySystem;
pub use self::collision_system::CollisionSystem;

use ecs::system::{ LazySystem, EntitySystem, InteractSystem };
use ecs::Entity;

use components::LevelComponents;

use util::TextureStore;

use nc::world::CollisionWorld;

mod render_system;
mod camera_system;
mod keyboard_system;
mod intent_system;
mod velocity_system;
mod gravity_system;
mod collision_system;

// TODO remove once nc::world::CollisionWorld2 is public
use na::{Pnt2, Iso2};
pub type CollisionWorld2<N, T> = CollisionWorld<Pnt2<N>, Iso2<N>, T>;

services! {
    struct LevelServices {
        texture_store: TextureStore = TextureStore::new_invalid(),
        collision_world: CollisionWorld2<f32, Entity> = CollisionWorld2::new(0.10, 0.10, false),
        delta_time_s: f32 = 0.0,
    }
}

systems! {
    struct LevelSystems<LevelComponents, LevelServices> {
        gravity_system: EntitySystem<GravitySystem> = EntitySystem::new(
            GravitySystem { g: 20.0 },
            aspect!(<LevelComponents> all: [gravity, velocity]),
            ),
        velocity_system: EntitySystem<VelocitySystem> = EntitySystem::new(
            VelocitySystem,
            aspect!(<LevelComponents> all: [velocity]),
            ),
        collision_system: EntitySystem<CollisionSystem> = EntitySystem::new(
            CollisionSystem::new(),
            aspect!(<LevelComponents> all: [position, collision]),
            ),
        render_system: LazySystem<InteractSystem<RenderSystem>> = LazySystem::new(),
        camera_system: EntitySystem<CameraSystem> = EntitySystem::new(
            CameraSystem::new(),
            aspect!(<LevelComponents> all: [camera])),
        keyboard_system: EntitySystem<KeyboardSystem> = EntitySystem::new(
            KeyboardSystem::new(),
            aspect!(<LevelComponents> all: [keyboard_input])),
        intent_system: EntitySystem<IntentSystem> = EntitySystem::new(
            IntentSystem,
            aspect!(<LevelComponents> all: [intents])),
    }
}

