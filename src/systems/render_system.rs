use ecs::{ Process, System, DataHelper };

use components::LevelComponents;

pub struct RenderSystem;

impl System for RenderSystem {
    type Components = LevelComponents;
    type Services = ();
}

impl Process for RenderSystem {
    fn process(&mut self, _: &mut DataHelper<LevelComponents, ()>) {
        println!("success!");
    }
}
