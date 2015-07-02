use ecs::{ Process, System, DataHelper };

use components::LevelComponents;

use glium::{self, Surface};

pub struct RenderSystem {
    pub display: glium::Display,
}

impl System for RenderSystem {
    type Components = LevelComponents;
    type Services = ();
}

impl Process for RenderSystem {
    fn process(&mut self, _: &mut DataHelper<LevelComponents, ()>) {
        let mut target = self.display.draw();
        target.clear_color(0.0, 0.0, 0.0, 0.0);
        target.finish().unwrap();
    }
}
