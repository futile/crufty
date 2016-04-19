use game::SpriteSheet;

#[derive(Debug, Copy, Clone Hash, PartialEq, Eq)]
pub struct SpriteSheetHandle(usize);

#[derive(Default)]
pub struct SpriteSheetStore {
    sprite_sheet_store: Vec<SpriteSheet>,
    handle_store: HashMap<PathBuf, SpriteSheetHandle>,
}

impl SpriteSheetStore {
    pub fn new() -> SpriteSheetStore {
        SpriteSheetStore::default()
    }

    pub fn get_sprite_sheet_info(&mut self, path: &Path) -> SpriteSheetHandle {
        if !self.handle_store.contains_key(path) {
            self.load_dir(path.parent().unwrap());
        }

        *self.handle_store.get(path).unwrap()
    }

    pub fn get_sprite_sheet(&self, handle: &SpriteSheetHandle) -> &SpriteSheet {
        self.sprite_sheet_store.get(handle.id).unwrap()
    }

    fn should_load(&self, path: &Path) -> bool {
        // TODO decide on file pattern for spritesheets
        // probably decide on a format first
        unimplemented!()
        // path.extension().map_or(false, |ext| ext == "png")
    }

    fn load_dir(&mut self, path: &Path) {
        if !path.is_dir() {
            panic!("Is not a directory: '{:?}'", path);
        }

        let mut file_paths = fs::read_dir(path)
                                 .unwrap()
                                 .map(|res| res.unwrap().path())
                                 .filter(|fpath| fpath.is_file())
                                 .filter(|fpath| self.should_load(fpath))
                                 .collect::<Vec<_>>();

        file_paths.sort();

        unimplemented!();

        // TODO actually load files
        let images =
            file_paths.iter()
                      .map(|fpath| image::open(fpath).unwrap().to_rgba())
                      .map(|image| {
                          let image_dimensions = image.dimensions();
                          glium::sprite_sheet::RawImage2d::from_raw_rgba_reversed(image.into_raw(),
                                                                             image_dimensions)
                      })
                      .collect::<Vec<_>>();

        println!("paths loaded: {:?}", file_paths);

        // TODO save to the store
        self.sprite_sheet_store
            .push(CompressedSrgbSprite_Sheet2dArray::new(self.display.as_ref().unwrap(), images)
                      .unwrap());
        let ss_id = self.sprite_sheet_store.len() - 1;

        for (idx, fpath) in file_paths.into_iter().enumerate() {
            let previous = self.handle_store.insert(fpath, SpriteSheetHandle(ss_id));

            assert_eq!(previous, None);
        }
    }
}
