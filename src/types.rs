use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::html_writer::ContentVec;
use crate::utils::get_content_ranges;
use crate::yaml;
use crate::yaml::{Yaml, YamlError};

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

#[derive(Debug, Eq, PartialEq)]
pub struct Page<'a> {
    _id: String,
    _slug: String,
    _directory: String,
    _path: PathBuf,
    _raw: &'a str,
    _html: String,
}

impl<'a> Page<'a> {
    pub fn write_json<P: AsRef<Path> + 'a, W: Write>(
        p: P,
        frontmatter: &'a str,
        raw: &'a str,
        content: &ContentVec,
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
        w.write_fmt(format_args!("_id: \"{}\", ", _id))?;
        w.write_fmt(format_args!("_slug: \"{}\", ", _slug.unwrap_or_default()))?;
        w.write_fmt(format_args!("_raw: {:?}, ", raw))?;
        w.write_fmt(format_args!(
            "_directory: \"{}\", ",
            _directory.unwrap_or_default()
        ))?;
        w.write_fmt(format_args!("_content: {}, ", content))?;
        if let Ok(yaml) = yaml {
            yaml.write_json(w)?;
        }
        w.write_all(" }".as_bytes())?;
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Token {
    path: (usize, usize),
    frontmatter: (usize, usize),
    body: (usize, usize),
}

#[derive(Default)]
pub struct Content {
    raw: String,
    tokens: Vec<Token>,
}

impl Content {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_capacity(size: usize) -> Self {
        Self {
            raw: String::with_capacity(size * 1100),
            tokens: Vec::with_capacity(size),
        }
    }
    pub fn push_file<P: AsRef<Path>>(&mut self, path: P, raw: &str) {
        let start = self.raw.len();
        let path_str = path.as_ref().to_string_lossy();
        let path = (start, path_str.len() + start);
        let start = start + path_str.len();
        self.raw.push_str(&path_str);
        let ranges = get_content_ranges(raw.as_bytes());
        self.raw.push_str(raw);
        let frontmatter = (
            ranges.frontmatter.start + start,
            ranges.frontmatter.end + start,
        );
        let body = (ranges.body.start + start, ranges.body.end + start);
        self.tokens.push(Token {
            path,
            frontmatter,
            body,
        });
    }
    pub fn len(&self) -> usize {
        self.tokens.len()
    }
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }
    pub fn tokens(&self) -> &[Token] {
        &self.tokens
    }
    pub fn path(&self, token: &Token) -> &str {
        &self.raw[token.path.0..token.path.1]
    }
    pub fn frontmatter_raw(&self, token: &Token) -> &str {
        &self.raw[token.frontmatter.0..token.frontmatter.1]
    }
    pub fn frontmatter<'a>(&'a self, token: &'a Token) -> Result<Yaml<'a>, YamlError> {
        yaml::Parser::from_str(self.frontmatter_raw(token)).parse()
    }
    pub fn body_raw(&self, token: &Token) -> &str {
        &self.raw[token.body.0..token.body.1]
    }
}
