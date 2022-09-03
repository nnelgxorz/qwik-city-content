use std::{
    fs::File,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{
    types::{Config, Page},
    utils::ContentRanges,
};

#[inline]
pub fn generate(
    ranges: ContentRanges,
    content: String,
    path: PathBuf,
    config: Arc<Config>,
) -> std::io::Result<()> {
    let path = path.to_str().unwrap();
    let input_str: String = config.input.display().to_string();
    if let Some(stripped) = path
        .strip_prefix(&input_str)
        .and_then(|s| s.strip_prefix('/'))
    {
        let outdir: &Path = &config.output;
        let output = outdir.join(format!("{}.tsx", &stripped));
        let data = match Page::create(
            &stripped,
            &output,
            &content[ranges.frontmatter.start..ranges.frontmatter.end],
            &content[ranges.body.start..ranges.body.end],
        ) {
            Some(data) => data,
            _ => return Ok(()),
        };
        if let Some(parent) = output.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let file = File::create(&output)?;
        let mut writer = BufWriter::new(file);
        let _ = writer.write(b"export default ")?;
        let _ = writer.write(serde_json::to_string(&data).unwrap().as_bytes())?;
        writer.flush()?;
    }
    Ok(())
}
