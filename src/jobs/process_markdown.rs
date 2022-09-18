use std::{
    fs::File,
    io::{BufWriter, Write},
    sync::Arc,
};

use crate::{
    html_writer::Markdown,
    types::{Config, Content, Page},
};

pub fn process_all(content: Arc<Content>, config: Arc<Config>) -> std::io::Result<()> {
    let mut html = String::new();
    let outdir = config.output.join("files");
    let input: String = config.input.to_string_lossy().to_string();
    for token in content
        .tokens()
        .iter()
        .filter(|t| content.path(t).ends_with(".md"))
    {
        let filename = content
            .path(token)
            .strip_prefix(&input)
            .unwrap()
            .trim_start_matches('/');
        let outpath = crate::utils::output_path(&outdir, filename);
        let file = File::create(outpath)?;
        let mut w = BufWriter::new(file);
        let path = content.path(token);
        let content_vec = match Markdown::new(config.clone()).write_md(content.body_raw(token)) {
            Ok(vec) => vec,
            Err(e) => {
                println!("{}", e);
                continue;
            }
        };
        // println!("{:?}", html_vec);
        let input_str: String = config.input.display().to_string();
        pulldown_cmark::html::push_html(
            &mut html,
            pulldown_cmark::Parser::new(content.body_raw(token)),
        );
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
            w.flush()?;
            html.clear();
        }
    }
    Ok(())
}
