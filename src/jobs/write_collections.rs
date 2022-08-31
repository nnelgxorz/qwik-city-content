use std::{
    io::{BufWriter, Write},
    sync::Arc,
};

use crate::{
    types::{Config, GeneratedData},
    utils::{camel_case, snake_case},
};

pub fn generate(config: Arc<Config>, gen: Arc<GeneratedData>) -> std::io::Result<()> {
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
        writer.write_fmt(format_args!(
            "export const {}: Tagged{}[] = [",
            snake_case(tag),
            camel_case(tag)
        ))?;
        if let Some(first) = id_iter.next() {
            writer.write_fmt(format_args!(" q{}", first))?;
            for id in id_iter {
                writer.write_fmt(format_args!(", q{}", id))?;
            }
        }
        let _ = writer.write(b"];\n")?;
    }

    let _ = writer.write(b"\n")?;

    for (tag, ids) in gen.collections.iter() {
        let mut id_iter = ids.iter();
        writer.write_fmt(format_args!(
            "export type Tagged{} = Merge<",
            camel_case(tag)
        ))?;
        if let Some(first) = id_iter.next() {
            writer.write_fmt(format_args!("typeof q{}", first))?;
            for id in id_iter {
                writer.write_fmt(format_args!(" | typeof q{}", id))?;
            }
        }
        let _ = writer.write(b">;\n")?;
    }
    Ok(())
}
