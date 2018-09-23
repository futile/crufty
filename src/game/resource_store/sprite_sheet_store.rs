use super::texture_store::TextureStore;

use crate::game::SpriteSheet;

use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct SpriteSheetHandle(usize);

#[derive(Default)]
pub struct SpriteSheetStore {
    sprite_sheets: Vec<SpriteSheet>,
    handles: HashMap<PathBuf, SpriteSheetHandle>,
}

impl SpriteSheetStore {
    pub fn new() -> SpriteSheetStore {
        SpriteSheetStore::default()
    }

    pub fn get_sprite_sheet_handle(&mut self,
                                   texture_store: &mut TextureStore,
                                   path: &Path)
                                   -> SpriteSheetHandle {
        if !self.handles.contains_key(path) {
            return self.load_sheet(texture_store, path);
        }

        *self.handles.get(path).unwrap()
    }

    pub fn get_sprite_sheet(&self, handle: SpriteSheetHandle) -> &SpriteSheet {
        self.sprite_sheets.get(handle.0).unwrap()
    }

    fn load_sheet(&mut self, texture_store: &mut TextureStore, path: &Path) -> SpriteSheetHandle {
        let sprite_sheet = load_sprite_sheet(texture_store, path);

        self.sprite_sheets.push(sprite_sheet);
        let ss_id = self.sprite_sheets.len() - 1;

        let previous = self.handles.insert(path.to_owned(), SpriteSheetHandle(ss_id));
        assert_eq!(previous, None);

        println!("sprite sheet loaded: {:?}", path);

        SpriteSheetHandle(ss_id)
    }
}

fn load_sprite_sheet(texture_store: &mut TextureStore, path: &Path) -> SpriteSheet {
    use crate::game::{SpriteSheet, Animation};

    use std::fs::File;
    use std::io::Read;

    use toml::{Value};

    let mut file_content = String::new();
    File::open(path).and_then(|mut f| f.read_to_string(&mut file_content)).unwrap();

    let res = file_content.parse::<Value>();

    if let Err(ref error) = res {
        println!("failed to parse TOML: {}", error);
    }
    // for err in &parser.errors {
    //     let (loline, locol) = parser.to_linecol(err.lo);
    //     let (hiline, hicol) = parser.to_linecol(err.hi);
    //     println!("{:?}:{}:{}-{}:{} error: {}",
    //              path,
    //              loline,
    //              locol,
    //              hiline,
    //              hicol,
    //              err.desc);
    // }

    let anim_toml = res.expect("sprite sheet parsing failed");

    let anim_toml = match anim_toml {
        Value::Table(tab) => tab,
        _ => panic!("loaded toml not a table, but: {}", anim_toml),
    };

    let mut animations: HashMap<String, Animation> = HashMap::new();

    for (name, value) in anim_toml {
        let anim_table = value.as_table().expect("<value> is not a table");

        let width = anim_table.get("width")
            .and_then(|val| val.as_float())
            .expect("width not found or not a float");
        let height = anim_table.get("height")
            .and_then(|val| val.as_float())
            .expect("height not found or not a float");
        let num_frames = anim_table.get("num-frames")
            .and_then(|val| val.as_integer())
            .expect("num-frames not found or not an integer");
        let sprite = anim_table.get("start-sprite")
            .and_then(|val| val.as_str())
            .expect("start-sprite not found or not a string");
        let durations = anim_table.get("durations").expect("durations not found");

        let vec_durations = match *durations {
            Value::Float(f) => vec![f as f32; num_frames as usize],
            Value::Array(ref v) => {
                v.iter()
                    .map(|v| v.as_float().expect("a duration is not a float") as f32)
                    .collect::<Vec<_>>()
            }
            _ => panic!("durations neither float nor array"),
        };

        // sprite path is relative to current folder of the spritesheet file
        let sprite_path = path.parent().unwrap().join(sprite);

        animations.insert(name.clone(),
                          Animation {
                              start_info: texture_store.get_texture_info(&sprite_path),
                              name: Rc::new(name),
                              num_frames: num_frames as u8,
                              frame_durations: vec_durations,
                              width: width as f32,
                              height: height as f32,
                          });
    }

    SpriteSheet::new(animations)
}
