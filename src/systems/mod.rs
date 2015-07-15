pub use self::render_system::{ RenderSystem, WorldViewport };
pub use self::camera_system::{ CameraSystem };

use ecs::system::{ LazySystem, EntitySystem, InteractSystem };

use components::LevelComponents;

use application::InputManager;

mod render_system;
mod camera_system;

services! {
    struct LevelServices {
        input_manager: InputManager = InputManager::new(),
    }
}

systems! {
    struct LevelSystems<LevelComponents, LevelServices> {
        render_system: LazySystem<InteractSystem<RenderSystem>> = LazySystem::new(),
        camera_system: EntitySystem<CameraSystem> = EntitySystem::new(
            CameraSystem::new(),
            aspect!(<LevelComponents> all: [camera])),
    }
}

