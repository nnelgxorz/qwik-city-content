use std::{
    fs::File,
    io::{BufWriter, Write},
    sync::Arc,
};

use crate::types::{Config, Content, Page};

pub fn process_all(content: Arc<Content>, config: Arc<Config>) -> std::io::Result<()> {
    let file = File::create(config.output.join("content.ts"))?;
    let mut writer = BufWriter::new(file);
    for (id, token) in content.tokens().iter().enumerate() {
        let path = content.path(token);
        let input_str: String = config.input.display().to_string();
        if let Some(stripped) = path
            .strip_prefix(&input_str)
            .and_then(|s| s.strip_prefix('/'))
        {
            writer.write_fmt(format_args!("export const q{} = ", id))?;
            Page::write(
                &stripped,
                content.frontmatter_raw(token),
                content.body_raw(token),
                &mut writer,
            )?;
            writer.write_all(b"\n")?;
        }
    }
    writer.flush()?;
    Ok(())
}
