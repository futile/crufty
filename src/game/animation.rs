use std::collections::HashMap;
use std::rc::Rc;

use crate::components::SpriteInfo;
use crate::game::TextureInfo;

#[derive(Clone, Debug)]
pub struct Animation {
    pub start_info: TextureInfo,
    pub name: Rc<String>,
    pub num_frames: u8,
    pub frame_durations: Vec<f32>, // in seconds
    pub width: f32,
    pub height: f32,
}

impl Animation {
    pub fn create_sprite_info(&self, frame_idx: u8) -> SpriteInfo {
        let mut new_tex_info = self.start_info;
        new_tex_info.idx += f32::from(frame_idx);

        SpriteInfo {
            width: self.width,
            height: self.height,
            texture_info: new_tex_info,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct SpriteSheet {
    animations: HashMap<String, Animation>,
}

impl SpriteSheet {
    pub fn new(animations: HashMap<String, Animation>) -> SpriteSheet {
        SpriteSheet {
            animations,
        }
    }

    pub fn get(&self, animation_name: &str) -> Option<&Animation> {
        self.animations.get(animation_name)
    }
}
