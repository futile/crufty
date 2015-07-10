pub use self::render_system::{ RenderSystem, WorldViewport };
pub use self::camera_system::{ CameraSystem };

use ecs::system::{ LazySystem, EntitySystem };

use components::LevelComponents;

mod render_system;
mod camera_system;

systems! {
    struct LevelSystems<LevelComponents, ()> {
        render_system: LazySystem<EntitySystem<RenderSystem>> = LazySystem::new(),
        camera_system: EntitySystem<CameraSystem> = EntitySystem::new(
            CameraSystem::new(),
            aspect!(<LevelComponents> all: [camera])),
    }
}

