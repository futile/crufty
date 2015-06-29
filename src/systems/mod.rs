pub use self::render_system::RenderSystem;

use ecs::system::LazySystem;

use components::LevelComponents;

mod render_system;

systems! {
    struct LevelSystems<LevelComponents, ()> {
        render_system: LazySystem<RenderSystem> = LazySystem::new()
    }
}

