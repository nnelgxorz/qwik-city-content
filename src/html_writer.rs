#![allow(dead_code)]
use pulldown_cmark::escape::{escape_href, escape_html};
use pulldown_cmark::{Alignment, CowStr};

use crate::utils::html_tag;
use crate::{imports::Imports, types::Config};
use std::collections::HashMap;
use std::fmt::{Display, Write as _};
use std::iter::Peekable;
// import without risk of name clashing
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum Content {
    Html(String),
    Component(Vec<String>),
}

impl Display for Content {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Content::Html(html) => f.write_fmt(format_args!("{:?}", html)),
            Content::Component(elements) => {
                for element in elements {
                    f.write_str(element.trim())?;
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug)]
pub struct ContentVec {
    inner: Vec<Content>,
}

impl Display for ContentVec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('[')?;
        let mut iter = self.inner.iter();
        if let Some(first) = iter.next() {
            f.write_fmt(format_args!("{}", first))?;
        }
        for item in iter {
            f.write_fmt(format_args!(", {}", item))?;
        }
        f.write_char(']')
    }
}

enum TableState {
    Head,
    Body,
}

pub struct Markdown<'a> {
    config: Arc<Config>,
    pub content: Vec<Content>,
    html_buffer: String,
    component_buffer: Vec<String>,
    table_state: TableState,
    table_alignments: Vec<Alignment>,
    table_cell_index: usize,
    numbers: HashMap<CowStr<'a>, usize>,
}

impl<'a> Markdown<'a> {
    pub fn new(config: Arc<Config>) -> Self {
        Self {
            config,
            content: Vec::default(),
            html_buffer: String::default(),
            component_buffer: Vec::default(),
            table_state: TableState::Head,
            table_alignments: vec![],
            table_cell_index: 0,
            numbers: HashMap::new(),
        }
    }
    pub fn reset(&mut self) {
        self.content.clear();
        self.html_buffer.clear();
        self.component_buffer.clear();
    }
    pub fn content(&mut self) -> Vec<Content> {
        let content = self.content.drain(..).collect();
        content
    }
    pub fn push_html_str(&mut self, string: &str) {
        self.html_buffer.push_str(string)
    }
    pub fn dump_html(&mut self) {
        if !self.html_buffer.is_empty() {
            let html = self.html_buffer.drain(..).collect();
            self.content.push(Content::Html(html))
        }
    }
    pub fn dump_component(&mut self) {
        if !self.component_buffer.is_empty() {
            let component = self.component_buffer.drain(..).collect();
            self.content.push(Content::Component(component))
        }
    }
    pub fn start_tag(&mut self, tag: pulldown_cmark::Tag<'a>) -> std::io::Result<()> {
        match tag {
            pulldown_cmark::Tag::Paragraph => {
                self.push_html_str("<p>");
            }
            pulldown_cmark::Tag::Heading(lvl, id, classes) => {
                self.push_html_str("<");
                self.push_html_str(&lvl.to_string());
                if let Some(id) = id {
                    self.push_html_str(" id=\"");
                    self.push_html_str(id);
                    self.html_buffer.push('"');
                }
                let mut classes = classes.iter();
                if let Some(class) = classes.next() {
                    self.push_html_str(" class=\"");
                    self.push_html_str(class);
                    for class in classes {
                        self.push_html_str(class);
                        self.html_buffer.push(' ');
                    }
                    self.html_buffer.push('"');
                }
                self.html_buffer.push('>')
            }
            pulldown_cmark::Tag::BlockQuote => self.push_html_str("<blockquote>"),
            pulldown_cmark::Tag::CodeBlock(info) => match info {
                pulldown_cmark::CodeBlockKind::Fenced(info) => {
                    let lang = info.split(' ').next().unwrap();
                    if lang.is_empty() {
                        self.push_html_str("<pre><code>")
                    } else {
                        self.push_html_str("<pre><code class=\"language-");
                        escape_html(&mut self.html_buffer, lang)?;
                        self.push_html_str("\">")
                    }
                }
                pulldown_cmark::CodeBlockKind::Indented => self.push_html_str("<pre><code>"),
            },
            pulldown_cmark::Tag::List(Some(1)) => self.push_html_str("<ol>"),
            pulldown_cmark::Tag::List(Some(start)) => {
                self.push_html_str("<ol start=\"");
                self.push_html_str(&start.to_string());
                self.html_buffer.push('>');
            }
            pulldown_cmark::Tag::List(None) => self.push_html_str("<ul>"),
            pulldown_cmark::Tag::Item => self.push_html_str("<li>"),
            pulldown_cmark::Tag::FootnoteDefinition(name) => {
                self.html_buffer
                    .push_str("<div class=\"footnote-definition\" id=\"");
                escape_html(&mut self.html_buffer, &*name)?;
                self.html_buffer
                    .push_str("\"><sup class=\"footnote-definition-label\">");
                let len = self.numbers.len() + 1;
                let number = *self.numbers.entry(name).or_insert(len);
                self.push_html_str(&number.to_string());
                self.push_html_str("</sup>")
            }
            pulldown_cmark::Tag::Table(alignments) => {
                self.table_alignments = alignments;
                self.push_html_str("<table>");
            }
            pulldown_cmark::Tag::TableHead => {
                self.table_state = TableState::Head;
                self.table_cell_index = 0;
                self.push_html_str("<thead><tr>")
            }
            pulldown_cmark::Tag::TableRow => {
                self.table_cell_index = 0;
                self.push_html_str("<tr>");
            }
            pulldown_cmark::Tag::TableCell => {
                match self.table_state {
                    TableState::Head => {
                        self.push_html_str("<th");
                    }
                    TableState::Body => {
                        self.push_html_str("<td");
                    }
                }
                match self.table_alignments.get(self.table_cell_index) {
                    Some(&Alignment::Left) => self.push_html_str(" style=\"text-align: left\">"),
                    Some(&Alignment::Center) => {
                        self.push_html_str(" style=\"text-align: center\">")
                    }
                    Some(&Alignment::Right) => self.push_html_str(" style=\"text-align: right\">"),
                    _ => self.html_buffer.push('>'),
                }
            }
            pulldown_cmark::Tag::Emphasis => self.push_html_str("<em>"),
            pulldown_cmark::Tag::Strong => self.push_html_str("<strong>"),
            pulldown_cmark::Tag::Strikethrough => self.push_html_str("<del>"),
            pulldown_cmark::Tag::Link(pulldown_cmark::LinkType::Email, dest, title) => {
                self.push_html_str("<a href=\"mailto:");
                escape_href(&mut self.html_buffer, &dest)?;
                if !title.is_empty() {
                    self.push_html_str("\" title=\"");
                    escape_html(&mut self.html_buffer, &title)?;
                }
                self.push_html_str("\">")
            }
            pulldown_cmark::Tag::Link(_link_type, dest, title) => {
                self.push_html_str("<a href=\"");
                escape_href(&mut self.html_buffer, &dest)?;
                if !title.is_empty() {
                    self.push_html_str("\" title=\"");
                    escape_html(&mut self.html_buffer, &title)?;
                }
                self.push_html_str("\">")
            }
            pulldown_cmark::Tag::Image(_link_type, dest, title) => {
                self.push_html_str("<img src=\"");
                escape_href(&mut self.html_buffer, &dest)?;
                self.push_html_str("\" alt=\"\"");
                // self.raw_text()?;
                if !title.is_empty() {
                    self.push_html_str("\" title=\"");
                    escape_html(&mut self.html_buffer, &title)?;
                }
                self.push_html_str("\" />")
            }
        }
        Ok(())
    }
    pub fn end_tag(&mut self, tag: pulldown_cmark::Tag) -> std::io::Result<()> {
        match tag {
            pulldown_cmark::Tag::Paragraph => self.push_html_str("</p>"),
            pulldown_cmark::Tag::Heading(lvl, _, _) => {
                self.push_html_str("</");
                let _ = write!(self.html_buffer, "{}", lvl);
                self.html_buffer.push('>')
            }
            pulldown_cmark::Tag::BlockQuote => self.push_html_str("</blockquote>"),
            pulldown_cmark::Tag::CodeBlock(_) => self.push_html_str("</code></pre>"),
            pulldown_cmark::Tag::List(Some(_)) => self.push_html_str("</ol>"),
            pulldown_cmark::Tag::List(None) => self.push_html_str("</ul>"),
            pulldown_cmark::Tag::Item => self.push_html_str("</li>"),
            pulldown_cmark::Tag::FootnoteDefinition(_) => self.push_html_str("</div>"),
            pulldown_cmark::Tag::Table(_) => self.push_html_str("</tbody></table>"),
            pulldown_cmark::Tag::TableHead => {
                self.push_html_str("</tr></thead><tbody>");
                self.table_state = TableState::Body;
            }
            pulldown_cmark::Tag::TableRow => self.push_html_str("</tr>"),
            pulldown_cmark::Tag::TableCell => {
                match self.table_state {
                    TableState::Head => {
                        self.push_html_str("</th>");
                    }
                    TableState::Body => {
                        self.push_html_str("</td>");
                    }
                }
                self.table_cell_index += 1;
            }
            pulldown_cmark::Tag::Emphasis => self.push_html_str("</em>"),
            pulldown_cmark::Tag::Strong => self.push_html_str("</strong>"),
            pulldown_cmark::Tag::Strikethrough => self.push_html_str("</del>"),
            pulldown_cmark::Tag::Link(_, _, _) => self.push_html_str("</a>"),
            pulldown_cmark::Tag::Image(_, _, _) => {}
        }
        Ok(())
    }
    pub fn write_mdx(&'a mut self, src: &'a str, imports: &Imports) -> std::io::Result<ContentVec> {
        let mut parser: Peekable<pulldown_cmark::Parser<'a, 'a>> =
            pulldown_cmark::Parser::new(src).into_iter().peekable();
        while let Some(event) = parser.next() {
            match event {
                pulldown_cmark::Event::Start(tag) => self.start_tag(tag)?,
                pulldown_cmark::Event::End(tag) => self.end_tag(tag)?,
                pulldown_cmark::Event::Text(string) => self.push_html_str(&string),
                pulldown_cmark::Event::Code(text) => {
                    self.push_html_str("<code>");
                    escape_html(&mut self.html_buffer, &text)?;
                    self.push_html_str("</code>");
                }
                pulldown_cmark::Event::Html(tag) => {
                    let tag_name = html_tag(&tag);
                    if imports.is_import(tag_name) {
                        self.dump_html();
                        self.component_buffer.push(tag.to_string());
                        loop {
                            match parser.peek() {
                                Some(pulldown_cmark::Event::Html(tag)) => {
                                    self.component_buffer.push(tag.to_string());
                                    parser.next();
                                }
                                Some(pulldown_cmark::Event::HardBreak)
                                | Some(pulldown_cmark::Event::SoftBreak) => {
                                    parser.next();
                                }
                                _ => {
                                    break;
                                }
                            }
                        }
                        self.dump_component();
                    } else {
                        self.push_html_str(&tag)
                    }
                }
                pulldown_cmark::Event::FootnoteReference(name) => {
                    let len = self.numbers.len() + 1;
                    self.html_buffer
                        .push_str("<sup class=\"footnote-reference\"><a href=\"#");
                    escape_html(&mut self.html_buffer, &name)?;
                    self.push_html_str("\">");
                    let number = *self.numbers.entry(name).or_insert(len);
                    self.push_html_str(&number.to_string());
                    self.push_html_str("</a></sup>");
                }
                pulldown_cmark::Event::SoftBreak => {}
                pulldown_cmark::Event::HardBreak => self.push_html_str("<br/>"),
                pulldown_cmark::Event::Rule => self.push_html_str("<hr/>"),
                pulldown_cmark::Event::TaskListMarker(true) => self
                    .html_buffer
                    .push_str("<input disabled=\"\" type=\"checkbox\" checked=\"\"/>"),
                pulldown_cmark::Event::TaskListMarker(false) => self
                    .html_buffer
                    .push_str("<input disabled=\"\" type=\"checkbox\" />"),
            }
        }
        self.dump_html();
        let content = self.content();
        self.reset();
        Ok(ContentVec { inner: content })
    }
    pub fn write_md(&'a mut self, src: &'a str) -> std::io::Result<ContentVec> {
        let parser = pulldown_cmark::Parser::new(src);
        for event in parser {
            match event {
                pulldown_cmark::Event::Start(tag) => self.start_tag(tag)?,
                pulldown_cmark::Event::End(tag) => self.end_tag(tag)?,
                pulldown_cmark::Event::Text(string) => self.push_html_str(&string),
                pulldown_cmark::Event::Code(text) => {
                    self.push_html_str("<code>");
                    escape_html(&mut self.html_buffer, &text)?;
                    self.push_html_str("</code>");
                }
                pulldown_cmark::Event::Html(html) => self.push_html_str(&html),
                pulldown_cmark::Event::FootnoteReference(name) => {
                    let len = self.numbers.len() + 1;
                    self.html_buffer
                        .push_str("<sup class=\"footnote-reference\"><a href=\"#");
                    escape_html(&mut self.html_buffer, &name)?;
                    self.push_html_str("\">");
                    let number = *self.numbers.entry(name).or_insert(len);
                    self.push_html_str(&number.to_string());
                    self.push_html_str("</a></sup>");
                }
                pulldown_cmark::Event::SoftBreak => {}
                pulldown_cmark::Event::HardBreak => self.push_html_str("<br/>"),
                pulldown_cmark::Event::Rule => self.push_html_str("<hr/>"),
                pulldown_cmark::Event::TaskListMarker(true) => self
                    .html_buffer
                    .push_str("<input disabled=\"\" type=\"checkbox\" checked=\"\"/>"),
                pulldown_cmark::Event::TaskListMarker(false) => self
                    .html_buffer
                    .push_str("<input disabled=\"\" type=\"checkbox\" />"),
            }
        }
        self.dump_html();
        let content = self.content();
        self.reset();
        Ok(ContentVec { inner: content })
    }
}

#[cfg(test)]
mod test {
    use std::{path::PathBuf, sync::Arc};

    use crate::types::Config;

    use super::Markdown;

    #[test]
    fn markdown() {
        let src = "# A heading\n\nSome text. ![a link](http://www.fake.com)";
        let config = Arc::new(Config::new(PathBuf::new(), PathBuf::new(), PathBuf::new()));
        let mut markdown = Markdown::new(config);
        let content = markdown.write_md(src).unwrap();
        println!("{}", content);
    }
}
