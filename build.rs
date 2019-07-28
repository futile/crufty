use std::env;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Write;
use std::collections::BTreeMap;
use std::cmp::Ordering;

use walkdir::{WalkDir, DirEntry};

fn files_before_dirs(d1: &DirEntry, d2: &DirEntry) -> Ordering {
    match d1.file_type() {
        d1d if d1d.is_dir() => {
            match d2.file_type() {
                d2d if d2d.is_dir() => Ordering::Equal,
                d2f if d2f.is_file() => Ordering::Greater,
                d2o => panic!("unexpected d2o file-type: {:?}", d2o),
            }
        }
        d1f if d1f.is_file() => {
            match d2.file_type() {
                d2d if d2d.is_dir() => Ordering::Less,
                d2f if d2f.is_file() => Ordering::Equal,
                d2o => panic!("unexpected d2o file-type: {:?}", d2o),
            }
        }
        d1o => {
            panic!("unexpected d1o file-type: {:?}", d1o);
        }
    }
}

fn path_to_slug_name(path: &Path) -> String {
    path.iter()
        .map(|os| os.to_str().unwrap())
        .collect::<Vec<&str>>()
        .join("__")
}

fn path_to_string(path: &Path) -> String {
    path.iter()
        .map(|os| os.to_str().unwrap())
        .collect::<Vec<&str>>()
        .join("/")
}

fn build_texture_slugs(out_file: &Path) {
    println!("[build_texture_slugs] out_file: '{}'", out_file.display());

    #[derive(Debug)]
    struct SlugData {
        id: usize,
        idx: u16,
        path: PathBuf,
    }

    let mut slug_map: BTreeMap<String, SlugData> = BTreeMap::new();

    let search_path = Path::new("./assets/textures/sprites");

    let walker = WalkDir::new(search_path).sort_by(|f1, f2| {
        files_before_dirs(f1, f2).then_with(|| f1.file_name().cmp(f2.file_name()))
    });
    fn should_visit(d: &DirEntry) -> bool {
        !d.file_type().is_file() || d.file_name().to_str().unwrap().ends_with(".png")
    }

    let mut cur_id: usize = 0;
    let mut cur_idx: u16 = 0;
    for entry in walker.into_iter().filter_entry(should_visit) {
        let entry = entry.unwrap();
        let entry_path = entry.path().strip_prefix(search_path).unwrap();
        // println!("[build_texture_slugs] {}", entry_path.display());

        match entry.file_type() {
            d if d.is_dir() => {
                cur_id += 1;
                cur_idx = 0;
                // println!("[build_texture_slugs] new cur_id: {}", cur_id);
            }
            f if f.is_file() => {
                let slug_name = path_to_slug_name(&entry_path.with_file_name(entry_path.file_stem().unwrap()));
                // println!("[build_texture_slugs] slug_name: '{}'", slug_name);
                slug_map.insert(slug_name, SlugData {
                    id: cur_id,
                    idx: cur_idx,
                    path: entry.path().to_path_buf(),
                });
                cur_idx += 1;
            }
            o => {
                panic!("unexpected o file-type: {:?}", o)
            }
        }

        println!("cargo:rerun-if-changed={}", entry.path().display());
    }

    println!("[build_texture_slugs] slug_map:\n{:#?}", slug_map);


    let mut f = File::create(out_file).unwrap();
    f.write_all(b"use std::path::Path;
use crate::game::{TextureInfo};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum TextureSlug {
").unwrap();

    for slug_name in slug_map.keys() {
        f.write_fmt(format_args!("  {},\n", slug_name)).unwrap();
    }
    f.write_all(b"}

impl TextureSlug {
  pub fn id(self) -> usize {
    match self {
").unwrap();

    for (slug_name, slug) in &slug_map {
        f.write_fmt(format_args!("      TextureSlug::{} => {},\n", slug_name, slug.id)).unwrap();
    }

    f.write_all(b"    }
  }

  pub fn idx(self) -> u16 {
    match self {
").unwrap();

    for (slug_name, slug) in &slug_map {
        f.write_fmt(format_args!("      TextureSlug::{} => {},\n", slug_name, slug.idx)).unwrap();
    }

    f.write_all(b"    }
  }

  pub fn texture_info(self) -> TextureInfo {
    match self {
").unwrap();

    for (slug_name, slug) in &slug_map {
        f.write_fmt(format_args!("      TextureSlug::{} => TextureInfo::new({}, {}.0f32),\n", slug_name, slug.id, slug.idx)).unwrap();
    }

    f.write_all(b"    }
  }

  pub fn path(self) -> &'static Path {
    match self {
").unwrap();

    for (slug_name, slug) in &slug_map {
        f.write_fmt(format_args!("      TextureSlug::{} => Path::new(\"{}\"),\n", slug_name, path_to_string(&slug.path))).unwrap();
    }

    f.write_all(b"    }
  }
}").unwrap();
}

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    build_texture_slugs(&out_dir.join("texture_slugs.rs"));
}
