#![allow(dead_code)]
#[derive(Default)]
pub struct Imports<'a> {
    imports: Vec<&'a str>,
    star_imports: Vec<&'a str>,
}

impl<'a> Imports<'a> {
    pub fn len(&self) -> usize {
        self.imports.len() + self.star_imports.len()
    }
    pub fn push_import(&mut self, import: &'a str) {
        self.imports.push(import)
    }
    pub fn push_star_import(&mut self, import: &'a str) {
        self.star_imports.push(import)
    }
    pub fn is_import(&self, str: &str) -> bool {
        if self.imports.contains(&str) {
            return true;
        }
        for import in self.star_imports.iter() {
            if import == &str
                || str.starts_with(import) && str.chars().nth(import.len()) == Some('.')
            {
                return true;
            }
        }
        false
    }
}

pub struct Parser<'a> {
    src: &'a str,
    imports: Vec<&'a str>,
    star_imports: Vec<&'a str>,
    lines: usize,
    parsed_len: usize,
}

impl<'a> Parser<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            src,
            imports: Vec::default(),
            star_imports: Vec::default(),
            lines: 0,
            parsed_len: 0,
        }
    }
    pub fn parse(mut self) -> Result<(Imports<'a>, usize), ()> {
        let lines = self.src.lines();
        let mut imports = Imports::default();
        for line in lines {
            if !line.starts_with("import") && !line.is_empty() {
                break;
            }
            let words = line.split_ascii_whitespace();
            match words.into_iter().nth(1) {
                None => continue,
                Some("*") => {
                    if let Some(import) = line.split_ascii_whitespace().into_iter().nth(3) {
                        imports.push_star_import(import)
                    }
                }
                Some(word) => {
                    let l_curly = line.find('{');
                    let r_curly = line.find('}');
                    match l_curly.zip(r_curly) {
                        None => {
                            imports.push_import(word);
                        }
                        Some((start, end)) => {
                            if let Some(slice) = line.get(start + 1..end) {
                                for import in slice.split(',').filter(|s| !s.trim().is_empty()) {
                                    imports.push_import(import.trim());
                                }
                            }
                        }
                    }
                }
            }
            self.lines += 1;
            self.parsed_len += line.len();
        }
        Ok((
            imports,
            std::cmp::min(self.src.len(), self.lines + self.parsed_len),
        ))
    }
}

#[cfg(test)]
mod test {
    use super::Parser;

    #[test]
    pub fn no_text() {
        let src = "";
        let (imports, body_start) = Parser::new(src).parse().unwrap();
        assert_eq!(imports.len(), 0);
        assert_eq!(body_start, 0)
    }
    #[test]
    pub fn no_imports() {
        let src = "No imports here.";
        let (imports, body_start) = Parser::new(src).parse().unwrap();
        assert_eq!(imports.len(), 0);
        assert_eq!(body_start, 0)
    }
    #[test]
    pub fn default_import() {
        let src = "import SomeComponent from \"./some-file\"";
        let (imports, body_start) = Parser::new(src).parse().unwrap();
        assert_eq!(imports.len(), 1, "Should have parsed one import");
        assert_eq!(
            body_start,
            src.len(),
            "Should start at the start of next line"
        );
        assert!(imports.is_import("SomeComponent"));
        assert!(!imports.is_import("SomeComponent.thing"));
    }
    #[test]
    pub fn star_imports() {
        let src = "import * as SomeComponent from \"./some-file\"";
        let (imports, body_start) = Parser::new(src).parse().unwrap();
        assert_eq!(imports.len(), 1, "Should have parsed one import");
        assert_eq!(
            body_start,
            src.len(),
            "Should start at the start of next line"
        );
        assert!(imports.is_import("SomeComponent"));
        assert!(imports.is_import("SomeComponent.thing"));
    }
    #[test]
    pub fn multiple_imports() {
        let src = "import { ComponentA, ComponentB, ComponentC } from \"./some-file\"";
        let (imports, body_start) = Parser::new(src).parse().unwrap();
        assert_eq!(imports.len(), 3, "Should have parsed one import");
        assert_eq!(
            body_start,
            src.len(),
            "Should start at the start of next line"
        );
        assert!(imports.is_import("ComponentA"));
        assert!(imports.is_import("ComponentB"));
        assert!(imports.is_import("ComponentC"));
    }
    #[test]
    pub fn multiple_imports_no_spaces() {
        let src = "import {ComponentA,ComponentB,ComponentC} from \"./some-file\"";
        let (imports, body_start) = Parser::new(src).parse().unwrap();
        assert_eq!(imports.len(), 3, "Should have parsed one import");
        assert_eq!(
            body_start,
            src.len(),
            "Should start at the start of next line"
        );
        assert!(imports.is_import("ComponentA"));
        assert!(imports.is_import("ComponentB"));
        assert!(imports.is_import("ComponentC"));
    }
    #[test]
    pub fn empty_multiple_imports_no_spaces() {
        let src = "import {} from \"./some-file\"";
        let (imports, body_start) = Parser::new(src).parse().unwrap();
        assert_eq!(imports.len(), 0, "Should have parsed zero imports");
        assert_eq!(
            body_start,
            src.len(),
            "Should start at the start of next line"
        );
    }
    #[test]
    pub fn empty_multiple_imports_() {
        let src = "import { } from \"./some-file\"";
        let (imports, body_start) = Parser::new(src).parse().unwrap();
        assert_eq!(imports.len(), 0, "Should have parsed zero imports");
        assert_eq!(
            body_start,
            src.len(),
            "Should start at the start of next line"
        );
    }
}
