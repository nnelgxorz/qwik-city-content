#![allow(dead_code)]
use std::fmt::Write as _;
use std::io::BufWriter;
use std::{io::Write, sync::Arc};

use crate::html_writer::Markdown;
use crate::types::Page;
use crate::{
    imports::Imports,
    types::{Config, Content},
};

pub fn process_all(content: Arc<Content>, config: Arc<Config>) -> std::io::Result<()> {
    // Allocate a mutable string outside of the render loop.
    // We need a buffer to write html into that we can dump when we
    // hit a user imported component.
    let mut buffer = String::new();
    let outdir = config.output.join("files");
    let input: String = config.input.to_string_lossy().to_string();
    for token in content
        .tokens()
        .iter()
        .filter(|t| content.path(t).ends_with(".mdx"))
    {
        let (imports, body_start) = crate::imports::Parser::new(content.body_raw(token))
            .parse()
            .unwrap_or((Imports::default(), 0));
        let filename = content
            .path(token)
            .strip_prefix(&input)
            .unwrap()
            .trim_start_matches('/');
        let outpath = crate::utils::output_path(&outdir, filename);
        let file = std::fs::File::create(outpath)?;
        let mut w = BufWriter::new(file);
        let mut import_lines = content.body_raw(token)[..body_start].lines();
        if let Some(next) = import_lines.next() {
            w.write_all(next.trim().as_bytes())?;
            for import in import_lines {
                w.write_all(b";\n")?;
                w.write_all(import.trim().as_bytes())?;
            }
            w.write_all(b";\n\n")?;
        }
        let content_vec = match Markdown::new(config.clone())
            .write_mdx(&content.body_raw(token)[body_start..], &imports)
        {
            Ok(vec) => vec,
            Err(e) => {
                println!("{}", e);
                continue;
            }
        };
        let input_str: String = config.input.display().to_string();
        let path = content.path(token);
        if let Some(stripped) = path
            .strip_prefix(&input_str)
            .and_then(|s| s.strip_prefix('/'))
        {
            w.write_all(b"export default ")?;
            Page::write_json(
                &stripped,
                content.frontmatter_raw(token),
                content.body_raw(token),
                &content_vec,
                &mut w,
            )?;
            w.write_all(b"\n")?;
        }
        // Drain the buffer after every iteration
        buffer.clear();
    }
    Ok(())
}

pub fn write_component<W: Write>(
    body: &str,
    imports: &Imports,
    buffer: &mut String,
    w: &mut W,
) -> std::io::Result<()> {
    buffer.push_str("");
    w.write_all(b"export const Component = component$(() => {\n")?;
    w.write_all(b"  return (<>\n")?;
    for event in pulldown_cmark::Parser::new(body) {
        match event {
            pulldown_cmark::Event::Start(tag) => match tag {
                pulldown_cmark::Tag::Paragraph => buffer.push_str("<p>"),
                pulldown_cmark::Tag::Heading(level, id, classes) => {
                    buffer.push('<');
                    buffer.push_str(&level.to_string());
                    if let Some(id) = id {
                        buffer.push_str("id=\"");
                        let _ = write!(buffer, "{}", id);
                        buffer.push('"');
                    }
                    let mut classes = classes.iter();
                    if let Some(class) = classes.next() {
                        buffer.push_str("class=\"");
                        buffer.push_str(class);
                        for class in classes {
                            buffer.push(' ');
                            buffer.push_str(class)
                        }
                        buffer.push('"');
                    }
                    buffer.push('>')
                }
                pulldown_cmark::Tag::BlockQuote => {}
                pulldown_cmark::Tag::CodeBlock(_) => {}
                pulldown_cmark::Tag::List(_) => {}
                pulldown_cmark::Tag::Item => {}
                pulldown_cmark::Tag::FootnoteDefinition(_) => {}
                pulldown_cmark::Tag::Table(_) => {}
                pulldown_cmark::Tag::TableHead => {}
                pulldown_cmark::Tag::TableRow => {}
                pulldown_cmark::Tag::TableCell => {}
                pulldown_cmark::Tag::Emphasis => {}
                pulldown_cmark::Tag::Strong => {}
                pulldown_cmark::Tag::Strikethrough => {}
                pulldown_cmark::Tag::Link(_, _, _) => {}
                pulldown_cmark::Tag::Image(_, _, _) => {}
            },
            pulldown_cmark::Event::End(tag) => match tag {
                pulldown_cmark::Tag::Paragraph => buffer.push_str("</p>"),
                pulldown_cmark::Tag::Heading(level, _, _) => {
                    buffer.push_str("</");
                    buffer.push_str(&level.to_string());
                    buffer.push('>')
                }
                pulldown_cmark::Tag::BlockQuote => {}
                pulldown_cmark::Tag::CodeBlock(_) => {}
                pulldown_cmark::Tag::List(_) => {}
                pulldown_cmark::Tag::Item => {}
                pulldown_cmark::Tag::FootnoteDefinition(_) => {}
                pulldown_cmark::Tag::Table(_) => {}
                pulldown_cmark::Tag::TableHead => {}
                pulldown_cmark::Tag::TableRow => {}
                pulldown_cmark::Tag::TableCell => {}
                pulldown_cmark::Tag::Emphasis => {}
                pulldown_cmark::Tag::Strong => {}
                pulldown_cmark::Tag::Strikethrough => {}
                pulldown_cmark::Tag::Link(_, _, _) => {}
                pulldown_cmark::Tag::Image(_, _, _) => {}
            },
            pulldown_cmark::Event::Text(text) => buffer.push_str(&text),
            pulldown_cmark::Event::Code(_) => {}
            pulldown_cmark::Event::Html(html) => {
                let tag = html
                    .trim()
                    .trim_start_matches(&['<', '/'])
                    .trim_end_matches(&['>', '/']);
                if imports.is_import(tag) && !buffer.is_empty() {
                    w.write_fmt(format_args!(
                        "    <div class=\"qc-content\" dangerouslySetInnerHTML={:?}/>\n",
                        buffer
                    ))?;
                    buffer.clear();
                };
                w.write_fmt(format_args!("    {}", html))?;
            }

            pulldown_cmark::Event::FootnoteReference(_) => {}
            pulldown_cmark::Event::SoftBreak => buffer.push('\n'),
            pulldown_cmark::Event::HardBreak => buffer.push_str("\n\n"),
            pulldown_cmark::Event::Rule => {}
            pulldown_cmark::Event::TaskListMarker(_) => {}
        }
    }
    if !buffer.is_empty() {
        w.write_fmt(format_args!(
            "    <div class=\"qc-content\" dangerouslySetInnerHTML={:?}/>\n",
            buffer
        ))?;
    }
    w.write_all(b"  </>)\n")?;
    w.write_all(b"});")?;
    Ok(())
}
