pub use self::render_system::RenderSystem;

use ecs::System;

use components::LevelComponents;

mod render_system;

systems! {
    LevelSystems<LevelComponents, ()> {
        render_system: RenderSystem = RenderSystem
    }
}

