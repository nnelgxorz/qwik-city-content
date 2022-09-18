use std::path::{Iter, Path};

pub struct RouteParams<'a> {
    inner: Iter<'a>,
}

impl<'a> RouteParams<'a> {
    pub fn from_path(path: &'a Path) -> Self {
        return Self { inner: path.iter() };
    }
}

impl<'a> Iterator for RouteParams<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        for segment in self.inner.by_ref() {
            if let Some(param) = segment
                .to_string_lossy()
                .strip_prefix('[')
                .and_then(|s| s.strip_suffix(']'))
            {
                return Some(param.to_owned());
            }
        }
        None
    }
}
