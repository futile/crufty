use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::collections::BTreeMap;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::collections::hash_map::DefaultHasher;

use vec_map::VecMap;
use walkdir::{DirEntry, WalkDir};

fn files_before_dirs(d1: &DirEntry, d2: &DirEntry) -> Ordering {
    match d1.file_type() {
        d1d if d1d.is_dir() => match d2.file_type() {
            d2d if d2d.is_dir() => Ordering::Equal,
            d2f if d2f.is_file() => Ordering::Greater,
            d2o => panic!("unexpected d2o file-type: {:?}", d2o),
        },
        d1f if d1f.is_file() => match d2.file_type() {
            d2d if d2d.is_dir() => Ordering::Less,
            d2f if d2f.is_file() => Ordering::Equal,
            d2o => panic!("unexpected d2o file-type: {:?}", d2o),
        },
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
    let mut id_to_slugs: VecMap<Vec<String>> = VecMap::new();

    let search_path = Path::new("assets/textures");

    let walker = WalkDir::new(search_path).sort_by(|f1, f2| {
        files_before_dirs(f1, f2).then_with(|| f1.file_name().cmp(f2.file_name()))
    });

    fn should_visit(d: &DirEntry) -> bool {
        !d.file_type().is_file() || d.file_name().to_str().unwrap().ends_with(".png")
    }

    let mut cur_id: usize = 0;
    let mut cur_id_used: bool = false;
    let mut cur_idx: u16 = 0;
    for entry in walker.into_iter().filter_entry(should_visit) {
        let entry = entry.unwrap();
        let entry_path = entry.path().strip_prefix(search_path).unwrap();

        match entry.file_type() {
            d if d.is_dir() => {
                if cur_id_used {
                    cur_id += 1;
                    cur_idx = 0;
                    cur_id_used = false;
                }
            }
            f if f.is_file() => {
                let slug_name =
                    path_to_slug_name(&entry_path.with_file_name(entry_path.file_stem().unwrap()));
                slug_map.insert(
                    slug_name.clone(),
                    SlugData {
                        id: cur_id,
                        idx: cur_idx,
                        path: entry.path().to_path_buf(),
                    },
                );
                id_to_slugs.reserve_len(cur_id + 1);
                id_to_slugs
                    .entry(cur_id)
                    .or_insert_with(Vec::new)
                    .push(slug_name);
                cur_idx += 1;
                cur_id_used = true;
            }
            o => panic!("unexpected o file-type: {:?}", o),
        }

        println!("cargo:rerun-if-changed={}", entry.path().display());
    }

    println!("[build_texture_slugs] slug_map:\n{:#?}", slug_map);

    let mut enum_content = String::new();
    let mut id_content = String::new();
    let mut idx_content = String::new();
    let mut texture_info_content = String::new();
    let mut path_content = String::new();
    let mut from_path_content = String::new();
    let mut id_to_slugs_content = String::new();

    for (slug_name, slug) in &slug_map {
        let slug_path_str = path_to_string(&slug.path);

        enum_content.push_str(&format!("  {},\n", slug_name));
        id_content.push_str(&format!(
            "      TextureSlug::{} => {},\n",
            slug_name, slug.id
        ));
        idx_content.push_str(&format!(
            "      TextureSlug::{} => {},\n",
            slug_name, slug.idx
        ));
        texture_info_content.push_str(&format!(
            "      TextureSlug::{} => TextureInfo::new({}, {}.0f32),\n",
            slug_name, slug.id, slug.idx
        ));
        path_content.push_str(&format!(
            "      TextureSlug::{} => Path::new(\"{}\"),\n",
            slug_name,
            slug_path_str
        ));

        let mut h = DefaultHasher::new();
        slug.path.hash(&mut h);
        let hash = h.finish();
        from_path_content.push_str(&format!(
            "      {} => Some(TextureSlug::{}), // \"{}\"\n", hash, slug_name, slug_path_str
        ))
    }

    for (id, slug_names) in &id_to_slugs {
        id_to_slugs_content.push_str(&format!(
            "      {} => Some(&[\n", id
        ));
        for s in slug_names {
            id_to_slugs_content.push_str(&format!(
                "        TextureSlug::{},\n", s
            ));
        }
        id_to_slugs_content.push_str(&format!(
            "      ]),\n"
        ));
    }

    let mut f = File::create(out_file).unwrap();
    f.write_fmt(format_args!("// this file is auto generated from build.rs

use std::path::Path;
use std::hash::{{Hash, Hasher}};
use std::collections::hash_map::DefaultHasher;

use crate::game::TextureInfo;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum TextureSlug {{
{enum}
}}

impl TextureSlug {{
  pub fn id(self) -> usize {{
    match self {{
{id}
    }}
  }}

  pub fn idx(self) -> u16 {{
    match self {{
{idx}
    }}
  }}

  pub fn texture_info(self) -> TextureInfo {{
    match self {{
{texture_info}
    }}
  }}

  pub fn from_path(path: &Path) -> Option<TextureSlug> {{
    let mut h = DefaultHasher::new();
    path.hash(&mut h);
    let hash = h.finish();

    match hash {{
{from_path}
      _ => None,
    }}
  }}

  pub fn path(self) -> &'static Path {{
    match self {{
{path}
    }}
  }}

  pub fn all_with_id(id: usize) -> Option<&'static [TextureSlug]> {{
    match id {{
{id_to_slugs}
      _ => None,
    }}
  }}
}}",
        enum=enum_content.trim_end(),
        id=id_content.trim_end(),
        idx=idx_content.trim_end(),
        texture_info=texture_info_content.trim_end(),
        path=path_content.trim_end(),
        from_path=from_path_content.trim_end(),
        id_to_slugs=id_to_slugs_content.trim_end()))
        .unwrap();
}

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    build_texture_slugs(&out_dir.join("texture_slugs.rs"));
}
