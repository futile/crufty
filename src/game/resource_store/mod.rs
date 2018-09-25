pub use self::sprite_sheet_store::SpriteSheetHandle;
pub use self::texture_store::TextureInfo;

use self::sprite_sheet_store::SpriteSheetStore;
use self::texture_store::TextureStore;

use crate::game::SpriteSheet;

use glium;
use glium::texture::CompressedSrgbTexture2dArray;

use std::path::Path;

mod sprite_sheet_store;
mod texture_store;

#[derive(Default)]
pub struct ResourceStore {
    texture_store: TextureStore,
    sprite_sheet_store: SpriteSheetStore,
}

impl ResourceStore {
    pub fn new(display: glium::Display) -> ResourceStore {
        ResourceStore {
            texture_store: TextureStore::new(display),
            sprite_sheet_store: SpriteSheetStore::new(),
        }
    }

    pub fn load_texture(&mut self, path: &Path) -> TextureInfo {
        self.texture_store.get_texture_info(path)
    }

    pub fn get_texture(&self, tex_info: TextureInfo) -> &CompressedSrgbTexture2dArray {
        self.texture_store.get_texture(tex_info)
    }

    pub fn load_sprite_sheet(&mut self, path: &Path) -> SpriteSheetHandle {
        self.sprite_sheet_store
            .get_sprite_sheet_handle(&mut self.texture_store, path)
    }

    pub fn get_sprite_sheet(&self, handle: SpriteSheetHandle) -> &SpriteSheet {
        self.sprite_sheet_store.get_sprite_sheet(handle)
    }
}
