use std::{
    collections::HashMap,
    io::{BufWriter, Write},
    path::Path,
    sync::Arc,
};

use crate::{
    types::{Config, Content},
    utils::{write_camel_case, write_output_path, write_snake_case},
    yaml::YamlError,
};

pub fn process_all(content: Arc<Content>, config: Arc<Config>) -> Result<(), YamlError> {
    let mut collections: HashMap<String, Vec<usize>> = HashMap::with_capacity(content.len() * 3);
    for (idx, token) in content.tokens().iter().enumerate() {
        let frontmatter = content.frontmatter(token)?;
        for tag in frontmatter.get_tags() {
            if let Some(xs) = collections.get_mut(tag) {
                xs.push(idx);
            } else {
                collections.insert(tag.to_owned(), vec![idx]);
            }
        }
    }
    if let Err(e) = write(content.clone(), config, collections) {
        println!("Collections error: {}", e);
    }
    Ok(())
}

#[inline]
pub fn write(
    content: Arc<Content>,
    config: Arc<Config>,
    gen: HashMap<String, Vec<usize>>,
) -> std::io::Result<()> {
    if content.tokens().is_empty() {
        return Ok(());
    }
    std::fs::create_dir_all(&config.output)?;
    let file = std::fs::File::create(config.output.join("collections.ts"))?;
    let mut writer = BufWriter::new(file);

    let _ = writer.write(b"import type { Merge } from \"./generated-helpers\";\n")?;
    let input = config.input.to_string_lossy();
    for (idx, token) in content.tokens().iter().enumerate() {
        let path = content.path(token).strip_prefix(&*input).unwrap();
        let output_path = crate::utils::output_path(Path::new("./files/"), path);
        let lossy = output_path.to_string_lossy();
        writer.write_fmt(format_args!("import q{} from \"", idx))?;
        write_output_path(&*lossy, path, &mut writer)?;
        writer.write_all(b"\"\n")?;
    }
    let _ = writer.write(b"\n")?;

    for (tag, ids) in gen.iter() {
        let mut id_iter = ids.iter();
        let _ = writer.write("export const ".as_bytes())?;
        write_snake_case(tag, &mut writer)?;
        let _ = writer.write(": ".as_bytes())?;
        write_camel_case(tag, &mut writer)?;
        let _ = writer.write("[] = [".as_bytes())?;
        if let Some(first) = id_iter.next() {
            writer.write_fmt(format_args!(" q{}", first))?;
            for id in id_iter {
                writer.write_fmt(format_args!(", q{}", id))?;
            }
        }
        let _ = writer.write(b"];\n")?;
    }
    let _ = writer.write("export const all: All[] = [".as_bytes())?;
    writer.write_all(" q1".as_bytes())?;
    for id in 1..content.tokens().len() {
        writer.write_fmt(format_args!(", q{}", id))?;
    }
    writer.write_all(b"];\n")?;

    let _ = writer.write(b"\n")?;

    for (tag, ids) in gen.iter() {
        let mut id_iter = ids.iter();
        let _ = writer.write("export type ".as_bytes())?;
        write_camel_case(tag, &mut writer)?;
        let _ = writer.write(" = Merge<".as_bytes())?;
        if let Some(first) = id_iter.next() {
            writer.write_fmt(format_args!("typeof q{}", first))?;
            for id in id_iter {
                writer.write_fmt(format_args!(" | typeof q{}", id))?;
            }
        }
        let _ = writer.write(b">;\n")?;
    }
    let _ = writer.write("export type All = Merge<".as_bytes())?;
    writer.write_all("typeof q0".as_bytes())?;
    for id in 1..content.tokens().len() - 1 {
        writer.write_fmt(format_args!(" | typeof q{}", id))?;
    }
    let _ = writer.write(b">;\n")?;
    Ok(())
}
