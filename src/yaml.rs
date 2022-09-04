use std::{io::Write, iter::Peekable, ops::Range, str::Chars};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum YamlKind {
    // Collection of Key/Value pairs
    Object,
    // A Key in a Yaml Object
    Key,
    // Collection of Yaml
    List,

    // Primitives
    String,
    Bool,
    Number,
    Null,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct YamlNode {
    pub parent: usize,
    pub kind: YamlKind,
    range: (usize, usize),
}

#[allow(dead_code)]
impl YamlNode {
    pub fn range(&self) -> Range<usize> {
        self.range.0..self.range.1
    }
    pub fn slice<'a>(&self, src: &'a str) -> &'a str {
        &src[self.range()]
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Yaml<'a> {
    src: &'a str,
    inner: Vec<YamlNode>,
    tags: Option<usize>,
    draft: Option<usize>,
}

#[allow(dead_code)]
impl<'a> Yaml<'a> {
    pub fn inner(&self) -> &[YamlNode] {
        &self.inner
    }
    pub fn len(&self) -> usize {
        self.inner.len()
    }
    pub fn write_json<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        write_json_values_rec(&self.inner, self.src, w)?;
        Ok(())
    }
    pub fn is_draft(&self) -> bool {
        if let Some(node) = self.draft.and_then(|idx| self.inner.get(idx)) {
            if node.kind == YamlKind::Bool {
                match node.slice(self.src) {
                    "true" => return true,
                    "false" => return false,
                    _ => todo!("Non true/false bool value"),
                }
            } else {
                println!("Draft must be a boolean")
            }
        }
        false
    }
    pub fn get_tags(&'a self) -> Tags<'a> {
        if let Some((idx, node)) = self
            .tags
            .and_then(|idx| self.inner.get(idx).map(|n| (idx, n)))
        {
            if node.kind == YamlKind::List {
                return Tags::from_slice(
                    idx + 1,
                    self.src,
                    self.inner.get(idx + 1..).unwrap_or(&[]),
                );
            } else {
                println!("Tags must be a list")
            }
        }
        Tags::from_slice(0, "", &[])
    }
}

pub struct Parser<'a> {
    src: &'a str,
    chars: Peekable<Chars<'a>>,
    start: usize,
    curr: usize,
    nodes: Vec<YamlNode>,
    tags: Option<usize>,
    draft: Option<usize>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum YamlError {
    UnexpectedEndOfInput,
    EmptyString,
    ExpectedDigit,
    Expected(char),
}

impl<'a> Parser<'a> {
    pub fn from_str(src: &'a str) -> Self {
        let trimmed = src.trim();
        Self {
            start: 0,
            curr: 0,
            chars: trimmed.chars().peekable(),
            src: trimmed,
            nodes: Vec::default(),
            tags: None,
            draft: None,
        }
    }
    pub fn slice(&self) -> &str {
        &self.src[self.start..self.curr]
    }
    pub fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }
    pub fn chomp(&mut self) {
        if let Some(next) = self.chars.next() {
            self.curr += next.len_utf8()
        }
    }
    pub fn commit(&mut self) {
        self.start = self.curr
    }
    pub fn skip_ws(&mut self) -> usize {
        let mut count = 0;
        loop {
            match self.peek() {
                Some(' ') => {
                    count += 1;
                    self.chomp()
                }
                Some('\t') => {
                    count += 2;
                    self.chomp()
                }
                _ => break,
            }
        }
        count
    }
    pub fn push_node(&mut self, kind: YamlKind, parent: usize) -> usize {
        self.nodes.push(YamlNode {
            parent,
            kind,
            range: (self.start, self.curr),
        });
        self.nodes.len()
    }
    pub fn push_key(&mut self, parent: usize) -> usize {
        self.nodes.push(YamlNode {
            parent,
            kind: YamlKind::Key,
            range: (self.start, self.curr),
        });
        let id = self.nodes.len();
        match self.slice() {
            "tags" => self.tags = Some(id),
            "draft" => self.draft = Some(id),
            _ => {}
        }
        id
    }
    pub fn parse(mut self) -> Result<Yaml<'a>, YamlError> {
        self.parse_object(0)?;
        Ok(Yaml {
            src: self.src,
            inner: self.nodes,
            tags: self.tags,
            draft: self.draft,
        })
    }
    pub fn parse_object(&mut self, parent: usize) -> Result<(), YamlError> {
        // always expect a key
        self.commit();
        while self.peek().is_some() {
            let key = self.parse_key(parent)?;
            self.parse_value(key)?;
            self.skip_ws();
            while let Some('\r') | Some('\n') = self.peek() {
                self.chomp();
            }
        }
        Ok(())
    }
    fn parse_inline_list(&mut self, parent: usize) -> Result<(), YamlError> {
        assert_eq!(self.peek(), Some('['));
        self.chomp();
        self.skip_ws();
        self.commit();
        let parent = self.push_node(YamlKind::List, parent);
        loop {
            self.skip_ws();
            self.parse_value(parent)?;
            self.skip_ws();
            match self.peek() {
                None => break,
                Some(']') => {
                    self.chomp();
                    break;
                }
                Some(',') => {
                    self.chomp();
                    self.skip_ws();
                    self.commit();
                }
                Some(c) => todo!("Unhandled {:?} in inline list", c),
            }
        }
        self.commit();
        Ok(())
    }
    fn parse_inline_object(&mut self, parent: usize) -> Result<(), YamlError> {
        assert_eq!(self.peek(), Some('{'));
        self.chomp();
        self.skip_ws();
        self.commit();
        let parent = self.push_node(YamlKind::Object, parent);
        loop {
            match self.peek() {
                Some('\r') | Some('\n') | Some('}') | None => break,
                _ => {
                    self.commit();
                    let parent = self.parse_key(parent)?;
                    self.skip_ws();
                    self.parse_value(parent)?;
                    self.skip_ws();
                    if let Some(',') = self.peek() {
                        self.chomp();
                        self.skip_ws();
                    }
                }
            }
        }
        assert_eq!(Some('}'), self.peek());
        self.chomp();
        self.commit();
        Ok(())
    }
    fn parse_multiline_value(&mut self, parent: usize, indent: usize) -> Result<(), YamlError> {
        match self.peek() {
            Some('-') => self.parse_multiline_list_items(parent, indent),
            _ => self.parse_multiline_object(parent, indent),
        }
    }
    fn parse_multiline_list_items(
        &mut self,
        parent: usize,
        indent: usize,
    ) -> Result<(), YamlError> {
        self.commit();
        let parent = self.push_node(YamlKind::List, parent);
        loop {
            if !matches!(self.peek(), Some('-')) {
                break;
            }
            self.chomp();
            self.skip_ws();
            self.commit();
            self.parse_value(parent)?;
            self.skip_ws();
            match self.peek() {
                Some('\r') | Some('\n') => {
                    while let Some('\r') | Some('\n') = self.peek() {
                        self.chomp();
                    }
                    let new_indent = self.skip_ws();
                    if new_indent > indent {
                        self.parse_multiline_value(parent, new_indent)?;
                    }
                    if new_indent < indent {
                        break;
                    }
                }
                _ => break,
            }
            self.commit()
        }
        Ok(())
    }
    fn parse_key(&mut self, parent: usize) -> Result<usize, YamlError> {
        self.commit();
        while let Some(next) = self.peek() {
            if next == ':' {
                let key = self.slice();
                if key.is_empty() {
                    return Err(YamlError::EmptyString);
                }
                let parent_idx = self.push_key(parent);
                self.chomp(); // Skip colon
                self.skip_ws();
                self.commit();
                return Ok(parent_idx);
            }
            self.chomp()
        }
        Err(YamlError::Expected(':'))
    }
    fn parse_multiline_object(&mut self, parent: usize, indent: usize) -> Result<(), YamlError> {
        self.commit();
        let parent = self.push_node(YamlKind::Object, parent);
        loop {
            let parent = self.parse_key(parent)?;
            self.skip_ws();
            self.commit();
            self.parse_value(parent)?;
            self.skip_ws();
            match self.peek() {
                Some('\r') | Some('\n') => {
                    while let Some('\r') | Some('\n') = self.peek() {
                        self.chomp();
                    }
                    let new_indent = self.skip_ws();
                    if new_indent > indent {
                        self.parse_multiline_value(parent, new_indent)?;
                    }
                    if new_indent < indent {
                        break;
                    }
                }
                _ => break,
            }
        }
        Ok(())
    }
    fn parse_value(&mut self, parent: usize) -> Result<(), YamlError> {
        match self.peek() {
            None => Err(YamlError::UnexpectedEndOfInput),
            Some('[') => self.parse_inline_list(parent),
            Some('{') => self.parse_inline_object(parent),
            Some('\r') | Some('\n') => {
                while let Some('\r') | Some('\n') = self.peek() {
                    self.chomp();
                }
                let indent = self.skip_ws();
                self.parse_multiline_value(parent, indent)
            }
            Some('"') | Some('\'') => self.parse_quoted_string(parent),
            Some(char) => {
                if char.is_ascii_digit() || char == '-' {
                    self.parse_number(parent)
                } else {
                    self.parse_string_bool_null(parent)
                }
            }
        }
    }
    fn parse_number(&mut self, parent: usize) -> Result<(), YamlError> {
        let mut next = self.peek();
        if next == Some('-') {
            self.chomp();
            next = self.peek();
        }
        if next.map(|c| !c.is_ascii_digit()).unwrap_or(false) {
            return Err(YamlError::ExpectedDigit);
        }
        let mut is_decimal = false;
        loop {
            match self.peek() {
                Some('\r') | Some('\n') | Some(',') | Some(']') | Some('}') | None => {
                    self.push_node(YamlKind::Number, parent);
                    self.commit();
                    break;
                }
                Some(char) => {
                    if char == '.' {
                        if is_decimal {
                            return self.parse_string_bool_null(parent);
                        } else {
                            is_decimal = true;
                            self.chomp();
                            continue;
                        }
                    }
                    if !char.is_ascii_digit() {
                        return self.parse_string_bool_null(parent);
                    }
                    self.chomp();
                }
            }
        }
        Ok(())
    }
    fn parse_quoted_string(&mut self, parent: usize) -> Result<(), YamlError> {
        let quote = match self.peek() {
            Some('\'') => '\'',
            Some('"') => '"',
            _ => return Err(YamlError::Expected('"')),
        };
        self.chomp();
        loop {
            match self.peek() {
                Some(char) => {
                    if char == quote {
                        self.chomp();
                        self.push_node(YamlKind::String, parent);
                        self.commit();
                        break;
                    }
                    self.chomp()
                }
                None => return Err(YamlError::Expected(quote)),
            }
        }
        Ok(())
    }
    fn parse_string_bool_null(&mut self, parent: usize) -> Result<(), YamlError> {
        self.commit();
        loop {
            match self.peek() {
                Some(',') | Some('\r') | Some('\n') | Some(']') | Some('}') | None => {
                    let kind = match self.slice().trim_end() {
                        "" => return Err(YamlError::EmptyString),
                        "false" | "NO" | "true" | "YES" => YamlKind::Bool,
                        "NULL" => YamlKind::Null,
                        _ => YamlKind::String,
                    };
                    let mut real_curr = self.curr;
                    for c in self.slice().chars().rev() {
                        if !c.is_ascii_whitespace() {
                            break;
                        }
                        real_curr -= 1;
                    }
                    self.nodes.push(YamlNode {
                        parent,
                        kind,
                        range: (self.start, real_curr),
                    });
                    self.commit();
                    break;
                }
                _ => self.chomp(),
            }
        }
        Ok(())
    }
}

pub struct Tags<'a> {
    parent: usize,
    src: &'a str,
    state: u8,
    inner: std::slice::Iter<'a, YamlNode>,
}

impl<'a> Tags<'a> {
    pub fn from_slice(parent: usize, src: &'a str, nodes: &'a [YamlNode]) -> Self {
        Self {
            parent,
            src,
            state: 0,
            inner: nodes.iter(),
        }
    }
}

impl<'a> Iterator for Tags<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.state == 1 {
            return None;
        }
        loop {
            let next = self.inner.next()?;
            if next.parent < self.parent {
                self.state = 1;
                return None;
            }
            let slice = next
                .slice(self.src)
                .trim_start_matches(|c| c == '\'' || c == '"')
                .trim_end_matches(|c| c == '\'' || c == '"');
            let is_valid = slice.starts_with(|c: char| c.is_ascii_alphabetic())
                && slice
                    .find(|c: char| !c.is_ascii_alphanumeric() && !matches!(c, '-' | ' ' | '_'))
                    .is_none();
            if is_valid && next.kind == YamlKind::String {
                return Some(slice);
            }
        }
    }
}

#[inline]
pub fn write_json_values_rec<W: Write>(
    nodes: &[YamlNode],
    src: &str,
    w: &mut W,
) -> std::io::Result<usize> {
    let mut processed = 0;
    let mut nodes_iter = nodes.iter().enumerate();
    while let Some((idx, node)) = nodes_iter.next() {
        let id = idx + 1;
        match node.kind {
            YamlKind::Object => {
                w.write_all("{ ".as_bytes())?;
                if id < nodes.len() {
                    let end = nodes[idx + 1..]
                        .iter()
                        .position(|n| n.parent < id)
                        .map(|i| i + id)
                        .unwrap_or(nodes.len());
                    for _ in 0..write_json_values_rec(&nodes[id..end], src, w)? {
                        nodes_iter.next();
                        processed += 1;
                    }
                }
                w.write_all(" }".as_bytes())?;
            }
            YamlKind::Key => {
                let slice = node
                    .slice(src)
                    .trim_start_matches(|c| c == '"' || c == '\'')
                    .trim_end_matches(|c| c == '"' || c == '\'');
                w.write_fmt(format_args!("\"{}\": ", slice))?;
                if id < nodes.len() {
                    let end = nodes[idx + 1..]
                        .iter()
                        .position(|n| n.parent < id)
                        .map(|i| i + idx + 1)
                        .unwrap_or(nodes.len());
                    for _ in 0..write_json_values_rec(&nodes[id..end], src, w)? {
                        nodes_iter.next();
                        processed += 1;
                    }
                }
            }
            YamlKind::List => {
                w.write_all("[ ".as_bytes())?;
                if id < nodes.len() {
                    let end = nodes[id..]
                        .iter()
                        .position(|n| n.parent < id)
                        .map(|i| i + id)
                        .unwrap_or(nodes.len());
                    for _ in 0..write_json_values_rec(&nodes[id..end], src, w)? {
                        nodes_iter.next();
                        processed += 1;
                    }
                }
                w.write_all(" ]".as_bytes())?;
            }
            YamlKind::String => {
                let slice = node
                    .slice(src)
                    .trim_start_matches(|c| c == '"' || c == '\'')
                    .trim_end_matches(|c| c == '"' || c == '\'');
                w.write_fmt(format_args!("\"{}\"", slice))?;
            }
            YamlKind::Bool => w.write_all(node.slice(src).as_bytes())?,
            YamlKind::Number => w.write_all(node.slice(src).as_bytes())?,
            YamlKind::Null => w.write_all("undefined".as_bytes())?,
        }
        processed += 1;
        if processed < nodes.len() {
            w.write_all(", ".as_bytes())?;
        }
    }
    Ok(processed)
}

#[cfg(test)]
mod tests {

    use super::{write_json_values_rec, Parser, Yaml, YamlKind};

    fn expect_nodes(src: &str, yaml: Yaml, expected: Vec<(&str, YamlKind, usize)>) {
        assert_eq!(yaml.len(), expected.len());
        for (index, node) in yaml.inner().iter().enumerate() {
            let (slice, kind, parent) = expected.get(index).unwrap();
            assert_eq!(*slice, node.slice(src));
            assert_eq!(*kind, node.kind);
            assert_eq!(*parent, node.parent);
        }
    }

    #[test]
    fn it_parses_bool_true() {
        let src = "draft: true";
        let yaml = Parser::from_str(src).parse().unwrap();
        let expected = vec![("draft", YamlKind::Key, 0), ("true", YamlKind::Bool, 1)];
        expect_nodes(src, yaml, expected);
    }
    #[test]
    fn it_parses_bool_false() {
        let src = "draft: false";
        let yaml = Parser::from_str(src).parse().unwrap();
        let expected = vec![("draft", YamlKind::Key, 0), ("false", YamlKind::Bool, 1)];
        expect_nodes(src, yaml, expected);
    }
    #[test]
    fn it_parses_bool_null() {
        let src = "draft: NULL";
        let yaml = Parser::from_str(src).parse().unwrap();
        let expected = vec![("draft", YamlKind::Key, 0), ("NULL", YamlKind::Null, 1)];
        expect_nodes(src, yaml, expected);
    }
    #[test]
    fn it_parses_unquoted_string() {
        let src = "key: An unquoted string";
        let yaml = Parser::from_str(src).parse().unwrap();
        let expected = vec![
            ("key", YamlKind::Key, 0),
            ("An unquoted string", YamlKind::String, 1),
        ];
        expect_nodes(src, yaml, expected);
    }
    #[test]
    fn it_parses_quoted_string() {
        let src = "key: A quoted string";
        let yaml = Parser::from_str(src).parse().unwrap();
        let expected = vec![
            ("key", YamlKind::Key, 0),
            ("A quoted string", YamlKind::String, 1),
        ];
        expect_nodes(src, yaml, expected);
    }
    #[test]
    fn it_parses_a_number() {
        let src = "key: 42";
        let yaml = Parser::from_str(src).parse().unwrap();
        let expected = vec![("key", YamlKind::Key, 0), ("42", YamlKind::Number, 1)];
        expect_nodes(src, yaml, expected);
    }
    #[test]
    fn it_parses_a_negative_number() {
        let src = "key: -42";
        let yaml = Parser::from_str(src).parse().unwrap();
        let expected = vec![("key", YamlKind::Key, 0), ("-42", YamlKind::Number, 1)];
        expect_nodes(src, yaml, expected);
    }
    #[test]
    fn it_parses_a_decimal_number() {
        let src = "key: 42.0";
        let yaml = Parser::from_str(src).parse().unwrap();
        let expected = vec![("key", YamlKind::Key, 0), ("42.0", YamlKind::Number, 1)];
        expect_nodes(src, yaml, expected);
    }
    #[test]
    fn it_parses_a_negative_decimal_number() {
        let src = "key: -42.0";
        let yaml = Parser::from_str(src).parse().unwrap();
        let expected = vec![("key", YamlKind::Key, 0), ("-42.0", YamlKind::Number, 1)];
        expect_nodes(src, yaml, expected);
    }
    #[test]
    fn it_parses_multiple_key_values() {
        let src = ["first: -42.0", "second: false", "third: Hello world!"].join("\n");
        let yaml = Parser::from_str(&src).parse().unwrap();
        let expected = vec![
            ("first", YamlKind::Key, 0),
            ("-42.0", YamlKind::Number, 1),
            ("second", YamlKind::Key, 0),
            ("false", YamlKind::Bool, 3),
            ("third", YamlKind::Key, 0),
            ("Hello world!", YamlKind::String, 5),
        ];
        expect_nodes(&src, yaml, expected);
    }
    #[test]
    fn it_parses_inline_list() {
        let src = "key: [\"A\", false, 42, Hello World]";
        let yaml = Parser::from_str(src).parse().unwrap();
        let expected = vec![
            ("key", YamlKind::Key, 0),
            ("", YamlKind::List, 1),
            ("\"A\"", YamlKind::String, 2),
            ("false", YamlKind::Bool, 2),
            ("42", YamlKind::Number, 2),
            ("Hello World", YamlKind::String, 2),
        ];
        expect_nodes(src, yaml, expected)
    }
    #[test]
    fn it_parses_inline_object() {
        let src = "key: { a: \"A\", b: false, c: 42, d: Hello World }";
        let yaml = Parser::from_str(src).parse().unwrap();
        let expected = vec![
            ("key", YamlKind::Key, 0),
            ("", YamlKind::Object, 1),
            ("a", YamlKind::Key, 2),
            ("\"A\"", YamlKind::String, 3),
            ("b", YamlKind::Key, 2),
            ("false", YamlKind::Bool, 5),
            ("c", YamlKind::Key, 2),
            ("42", YamlKind::Number, 7),
            ("d", YamlKind::Key, 2),
            ("Hello World", YamlKind::String, 9),
        ];
        expect_nodes(src, yaml, expected)
    }
    #[test]
    fn it_parses_multiline_lists() {
        let src = vec!["key:", " - A", " - \"B\"", " - false", " - 42"].join("\n");
        let expected = vec![
            ("key", YamlKind::Key, 0),
            ("", YamlKind::List, 1),
            ("A", YamlKind::String, 2),
            ("\"B\"", YamlKind::String, 2),
            ("false", YamlKind::Bool, 2),
            ("42", YamlKind::Number, 2),
        ];
        let yaml = Parser::from_str(&src).parse().unwrap();
        expect_nodes(&src, yaml, expected);
    }
    #[test]
    fn it_parses_nested_multiline_lists() {
        let src = vec!["key:", " - A", " - \"B\"", "   - false", "   - 42"].join("\n");
        let expected = vec![
            ("key", YamlKind::Key, 0),
            ("", YamlKind::List, 1),
            ("A", YamlKind::String, 2),
            ("\"B\"", YamlKind::String, 2),
            ("", YamlKind::List, 2),
            ("false", YamlKind::Bool, 5),
            ("42", YamlKind::Number, 5),
        ];
        let yaml = Parser::from_str(&src).parse().unwrap();
        expect_nodes(&src, yaml, expected);
    }
    #[test]
    fn it_parses_multiline_objects() {
        let src = vec!["key:", "  a: \"A\"", "  b: B", "  c: false", "  d: 42"].join("\n");
        let yaml = Parser::from_str(&src).parse().unwrap();
        let expected = vec![
            ("key", YamlKind::Key, 0),
            ("", YamlKind::Object, 1),
            ("a", YamlKind::Key, 2),
            ("\"A\"", YamlKind::String, 3),
            ("b", YamlKind::Key, 2),
            ("B", YamlKind::String, 5),
            ("c", YamlKind::Key, 2),
            ("false", YamlKind::Bool, 7),
            ("d", YamlKind::Key, 2),
            ("42", YamlKind::Number, 9),
        ];
        expect_nodes(&src, yaml, expected);
    }
    #[test]
    fn parses_full_yaml() {
        let src = vec![
            "title: Some title",
            "description: \"A description\"",
            "draft: true",
            "navigation:",
            "  key: A Key",
            "  weight: 0",
            "tags: [\"fun\", \"qwik\", \"stuff\"]",
        ]
        .join("\n");
        let yaml = Parser::from_str(&src).parse().unwrap();
        for (idx, node) in yaml.inner().iter().enumerate() {
            println!(
                "{{id: {}, parent: {}, kind: {:?} }}",
                idx + 1,
                node.parent,
                node.kind,
            )
        }
        let mut out = Vec::default();
        write_json_values_rec(yaml.inner(), &src, &mut out).unwrap();
        println!("{}", String::from_utf8(out).unwrap())
    }
    #[test]
    fn it_can_be_draft() {
        let src = "draft: true";
        let yaml = Parser::from_str(src).parse().unwrap();
        assert!(yaml.is_draft())
    }
    #[test]
    fn it_can_have_tags() {
        let src = "tags: [\"fun\", \"qwik\", \"stuff\"]";
        let yaml = Parser::from_str(src).parse().unwrap();
        for tag in yaml.get_tags() {
            println!("{:?}", tag);
        }
    }
    #[test]
    fn it_writes_multiple_key_values() {
        let src = ["first: -42.0", "second: false", "third: Hello world!"].join("\n");
        let yaml = Parser::from_str(&src).parse().unwrap();
        let mut out: Vec<u8> = Vec::default();
        write_json_values_rec(yaml.inner(), &src, &mut out).unwrap();
        println!("{}", String::from_utf8(out).unwrap());
    }
}
