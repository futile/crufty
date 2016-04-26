use game::SpriteSheet;

use std::fs::File;
use std::path::Path;
use std::io::Read;

pub fn load_sprite_sheet(path: &Path) -> SpriteSheet {
    let mut file_content = String::new();
    File::open(path).and_then(|mut f| f.read_to_string(&mut file_content)).unwrap();

    unimplemented!()
}
