use std::{
    io::{BufWriter, Write},
    sync::Arc,
};

use crate::{
    types::{Config, GeneratedData},
    utils::{write_camel_case, write_snake_case},
};

#[inline]
pub fn generate(config: Arc<Config>, gen: Arc<GeneratedData>) -> std::io::Result<()> {
    if gen.output_paths.is_empty() {
        return Ok(());
    }
    std::fs::create_dir_all(&config.output)?;
    let file = std::fs::File::create(config.output.join("collections.ts"))?;
    let mut writer = BufWriter::new(file);

    let _ = writer.write(b"import type { Merge } from \"./generated-helpers\";\n")?;
    for (idx, path) in gen.output_paths.iter().enumerate() {
        writer.write_fmt(format_args!(
            "import q{} from \"./{}\";\n",
            idx,
            path.display()
        ))?;
    }

    let _ = writer.write(b"\n")?;

    for (tag, ids) in gen.collections.iter() {
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
    for id in 1..gen.collections.len() {
        writer.write_fmt(format_args!(", q{}", id))?;
    }
    writer.write_all(b"];\n")?;

    let _ = writer.write(b"\n")?;

    for (tag, ids) in gen.collections.iter() {
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
    for id in 1..gen.output_paths.len() - 1 {
        writer.write_fmt(format_args!(" | typeof q{}", id))?;
    }
    let _ = writer.write(b">;\n")?;
    Ok(())
}
