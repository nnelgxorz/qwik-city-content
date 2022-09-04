use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::Hasher;
use std::io::Write;
use std::path::{Path, PathBuf};

pub struct Config {
    pub input: PathBuf,
    pub output: PathBuf,
    pub routes: PathBuf,
}

impl Config {
    pub fn new(input: PathBuf, output: PathBuf, routes: PathBuf) -> Self {
        Self {
            input,
            output,
            routes,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Default)]
pub struct GeneratedData {
    pub output_paths: Vec<PathBuf>,
    pub taxonomies: HashMap<String, Vec<usize>>,
    pub collections: HashMap<String, Vec<usize>>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Page<'a> {
    _id: u64,
    _slug: String,
    _directory: String,
    _path: PathBuf,
    _out_path: &'a Path,
    _raw: &'a str,
    _html: String,
}

impl<'a> Page<'a> {
    pub fn write<P: AsRef<Path> + 'a, W: Write>(
        p: P,
        out_path: &'a Path,
        frontmatter: &'a str,
        raw: &'a str,
        w: &mut W,
    ) -> std::io::Result<()> {
        let mut hasher = DefaultHasher::new();
        hasher.write(p.as_ref().to_str().unwrap_or_default().as_bytes());
        let _id = hasher.finish();
        let _path = p.as_ref().to_path_buf();
        let _slug = _path.file_stem().and_then(|s| s.to_str());
        let _directory = _path.parent().and_then(|s| s.to_str());
        let yaml = crate::yaml::Parser::from_str(frontmatter).parse();
        w.write_all("{ ".as_bytes())?;
        w.write_fmt(format_args!("_id: {}, ", _id))?;
        w.write_fmt(format_args!("_slug: \"{}\", ", _slug.unwrap_or_default()))?;
        w.write_fmt(format_args!(
            "_directory: \"{}\", ",
            _directory.unwrap_or_default()
        ))?;
        w.write_all("_html: \"".as_bytes())?;
        pulldown_cmark::html::write_html(w.by_ref(), pulldown_cmark::Parser::new(raw))?;
        w.write_all(", \"".as_bytes())?;
        w.write_fmt(format_args!("_outpath: \"{}\", ", out_path.display()))?;
        if let Ok(yaml) = yaml {
            yaml.write_json(w)?;
        }
        w.write_all(" }".as_bytes())?;
        Ok(())
    }
}
