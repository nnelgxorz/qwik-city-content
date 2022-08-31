use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeMap, HashMap};
use std::hash::Hasher;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub struct Config {
    pub input: PathBuf,
    pub output: PathBuf,
}

impl Config {
    pub fn new(input: PathBuf, output: PathBuf) -> Self {
        Self { input, output }
    }
}

#[derive(Debug, PartialEq, Eq, Default)]
pub struct GeneratedData {
    pub output_paths: Vec<PathBuf>,
    pub taxonomies: HashMap<String, Vec<usize>>,
    pub collections: HashMap<String, Vec<usize>>,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct Page<'a> {
    _id: u64,
    _slug: String,
    _directory: String,
    _path: PathBuf,
    _out_path: &'a Path,
    _raw: &'a str,
    #[serde(flatten)]
    frontmatter: BTreeMap<String, serde_yaml::Value>,
    _html: String,
}

impl<'a> Page<'a> {
    pub fn create<P: AsRef<Path> + 'a>(
        p: P,
        out_path: &'a Path,
        frontmatter: &'a str,
        raw: &'a str,
    ) -> Option<Self> {
        let mut hasher = DefaultHasher::new();
        hasher.write(p.as_ref().to_str().unwrap_or_default().as_bytes());
        let _id = hasher.finish();
        let _path = p.as_ref().to_path_buf();
        let _slug = _path.file_stem().and_then(|s| s.to_str())?;
        let _directory = _path.parent().and_then(|s| s.to_str())?;
        let front_matter = serde_yaml::from_str(frontmatter).unwrap_or_else(|e| {
            println!("Err: {}\n - {}", _path.display(), e);
            BTreeMap::new()
        });
        let mut html = String::new();
        pulldown_cmark::html::push_html(&mut html, pulldown_cmark::Parser::new(raw));
        Some(Self {
            _id,
            _slug: _slug.to_owned(),
            _directory: _directory.to_owned(),
            _path,
            _out_path: out_path,
            _raw: raw.trim(),
            frontmatter: front_matter,
            _html: html,
        })
    }
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct FrontMatter {
    #[serde(default)]
    pub tags: Vec<String>,
}
