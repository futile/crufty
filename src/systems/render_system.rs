use ecs::{ Process, System, DataHelper };

use components::LevelComponents;

use glium;

pub struct RenderSystem {
    pub display: glium::Display,
}

impl System for RenderSystem {
    type Components = LevelComponents;
    type Services = ();
}

impl Process for RenderSystem {
    fn process(&mut self, _: &mut DataHelper<LevelComponents, ()>) {
        println!("success!");
    }
}
