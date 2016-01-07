use std::collections::{HashMap};

use std::path::{Path, PathBuf};
use std::fs::{self};

use image;

use glium::texture::CompressedSrgbTexture2dArray;
use glium;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TextureInfo {
    pub idx: f32,

    id: usize,
}

pub struct TextureStore {
    tex_store: Vec<CompressedSrgbTexture2dArray>,
    info_store: HashMap<PathBuf, TextureInfo>,

    display: Option<glium::Display>,
}

impl Default for TextureStore {
    fn default() -> TextureStore { TextureStore::new_invalid() }
}

impl TextureStore {
    pub fn new_invalid() -> TextureStore {
        TextureStore {
            tex_store: Vec::new(),
            info_store: HashMap::new(),
            display: None,
        }
    }

    pub fn new(display: glium::Display) -> TextureStore {
        TextureStore {
            tex_store: Vec::new(),
            info_store: HashMap::new(),
            display: Some(display),
        }
    }

    pub fn get_texture_info(&mut self, path: &Path) -> TextureInfo {
        if !self.info_store.contains_key(path) {
            self.load_dir(path.parent().unwrap());
        }

        *self.info_store.get(path).unwrap()
    }

    pub fn get_texture(&self, tex_info: &TextureInfo) -> &CompressedSrgbTexture2dArray {
        self.tex_store.get(tex_info.id).unwrap()
    }

    fn load_dir(&mut self, path: &Path) {
        if !path.is_dir() {
            panic!("Is not a directory: '{:?}'", path);
        }

        let mut file_paths = Vec::new();

        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();

            if !entry.path().is_dir() {
                file_paths.push(entry.path());
            }
        }

        file_paths.sort();

        let images = file_paths.iter()
            .filter(|fpath| !fpath.is_dir())
            .map(|fpath| image::open(fpath).unwrap().to_rgba())
            .map(|image| {
                let image_dimensions = image.dimensions();
                glium::texture::RawImage2d::from_raw_rgba_reversed(image.into_raw(), image_dimensions)
            })
            .collect::<Vec<_>>();

        println!("paths loaded: {:?}", file_paths);

        self.tex_store.push(CompressedSrgbTexture2dArray::new(self.display.as_ref().unwrap(), images).unwrap());
        let tex_id = self.tex_store.len() - 1;

        for (idx, fpath) in file_paths.into_iter().enumerate() {
            self.info_store.insert(fpath, TextureInfo { id: tex_id, idx: idx as f32});
        }
    }
}
