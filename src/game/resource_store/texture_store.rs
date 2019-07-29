use std::cell::{RefCell, RefMut};
use std::ops::Deref;

use image;

use vec_map::VecMap;

use glium;
use glium::texture::CompressedSrgbTexture2dArray;

use crate::resources::TextureSlug;

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TextureInfo {
    pub idx: f32,

    id: usize,
}

impl TextureInfo {
    pub fn new(id: usize, idx: f32) -> TextureInfo {
        TextureInfo {
            idx,
            id,
        }
    }
}

pub struct TextureStore {
    tex_store: RefCell<VecMap<CompressedSrgbTexture2dArray>>,
    display: Option<glium::Display>,
}

impl Default for TextureStore {
    fn default() -> TextureStore {
        TextureStore::new_invalid()
    }
}

impl TextureStore {
    pub fn new_invalid() -> TextureStore {
        TextureStore {
            tex_store: RefCell::default(),
            display: None,
        }
    }

    pub fn new(display: glium::Display) -> TextureStore {
        TextureStore {
            display: Some(display),
            ..TextureStore::new_invalid()
        }
    }

    pub fn get_texture(&self, tex_info: TextureInfo) -> impl Deref<Target=CompressedSrgbTexture2dArray> + '_ {
        RefMut::map(self.tex_store.borrow_mut(), |store| store.entry(tex_info.id).or_insert_with(|| {
            TextureStore::load_all_with_id(tex_info.id, self.display.as_ref().unwrap())
        }))
    }

    fn load_all_with_id(id: usize, display: &glium::Display) -> CompressedSrgbTexture2dArray {
        let slugs = TextureSlug::all_with_id(id).expect("unknown texture id");

        let images = slugs.iter()
            .map(|slug| image::open(slug.path()).unwrap().to_rgba())
            .map(|image| {
                let image_dimensions = image.dimensions();
                glium::texture::RawImage2d::from_raw_rgba_reversed(
                    &image.into_raw(),
                    image_dimensions,
                )
            })
            .collect::<Vec<_>>();

        println!("textures loaded for: {:?}", slugs);

        CompressedSrgbTexture2dArray::new(display, images).unwrap()
    }
}
