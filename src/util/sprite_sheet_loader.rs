use game::{SpriteSheet, Animation};
use util::TextureStore;

use std::fs::File;
use std::path::Path;
use std::io::Read;
use std::collections::HashMap;

use toml::{Parser, Value};

pub fn load_sprite_sheet(texture_store: &mut TextureStore, path: &Path) -> SpriteSheet {
    let mut file_content = String::new();
    File::open(path).and_then(|mut f| f.read_to_string(&mut file_content)).unwrap();

    let mut parser = Parser::new(&file_content);
    let res = parser.parse();

    for err in &parser.errors {
        let (loline, locol) = parser.to_linecol(err.lo);
        let (hiline, hicol) = parser.to_linecol(err.hi);
        println!("{:?}:{}:{}-{}:{} error: {}",
                 path,
                 loline,
                 locol,
                 hiline,
                 hicol,
                 err.desc);
    }

    let anim_toml = res.expect("sprite sheet parsing failed");

    // println!("toml: {:#?}", anim_toml);

    let mut animations: HashMap<String, Animation> = HashMap::new();

    for (name, value) in anim_toml {
        let anim_table = match value {
            Value::Table(ref t) => t,
            _ => panic!("{:?} is not a table!", value),
        };

        // println!("anim_table: {:#?}", anim_table);

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

        let vec_durations = match durations {
            &Value::Float(f) => vec![f as f32; num_frames as usize],
            &Value::Array(ref v) => {
                v.iter()
                 .map(|v| v.as_float().expect("a duration is not a float") as f32)
                 .collect::<Vec<_>>()
            }
            _ => panic!("durations neither float nor array"),
        };

        // sprite path is relative to current folder of the spritesheet file
        let sprite_path = path.parent().unwrap().join(sprite);

        animations.insert(name,
                          Animation {
                              start_info: texture_store.get_texture_info(&sprite_path),
                              num_frames: num_frames as u8,
                              frame_durations: vec_durations,
                              width: width as f32,
                              height: height as f32,
                          });
    }

    SpriteSheet::new(animations)
}

// #[cfg(test)]
// mod tests {
//     use super::load_sprite_sheet;
//     use std::path::Path;

//     #[test]
//     fn simple_example() {
//         load_sprite_sheet(Path::new("tests/animations.toml"));
//     }
// }
