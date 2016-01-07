use ecs::{System, DataHelper, EntityIter};
use ecs::system::EntityProcess;

use super::LevelServices;

// use systems::WorldViewport;

use components::LevelComponents;

pub struct CameraSystem {
    pub resized: Option<(u32, u32)>,
}

impl CameraSystem {
    pub fn new() -> CameraSystem {
        CameraSystem { resized: None }
    }
}

impl System for CameraSystem {
    type Components = LevelComponents;
    type Services = LevelServices;

    // make system passive, so we have to call it manually
    fn is_active(&self) -> bool {
        false
    }
}

impl EntityProcess for CameraSystem {
    fn process(&mut self,
               entities: EntityIter<LevelComponents>,
               data: &mut DataHelper<LevelComponents, LevelServices>) {
        if let Some((win_width, win_height)) = self.resized {
            self.resized = None;

            for e in entities {
                let camera = &mut data.camera[e];

                if !camera.resize_world_to_window {
                    continue;
                }

                let svp = &camera.screen_viewport;

                let view_width_pc = (svp.maxs().x - svp.mins().x) / 2.0;
                let view_height_pc = (svp.maxs().y - svp.mins().y) / 2.0;

                camera.world_viewport.width = view_width_pc * (win_width as f32);
                camera.world_viewport.height = view_height_pc * (win_height as f32);
            }
        }
    }
}
