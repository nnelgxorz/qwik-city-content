use std::{
    io::{BufWriter, Write},
    path::Path,
};

use crate::route_params::RouteParams;

#[inline]
pub fn generate<P: AsRef<Path>>(routes: P) -> std::io::Result<usize> {
    let mut count = 0;
    generate_route_params_rec(routes, &mut count)?;
    Ok(count)
}

fn generate_route_params_rec<P: AsRef<Path>>(root: P, count: &mut usize) -> std::io::Result<()> {
    let dir = std::fs::read_dir(root)?;
    for entry in dir.filter_map(|e| e.ok()) {
        if entry.path().is_dir() {
            generate_route_params_rec(entry.path(), count)?;
        }
        if entry.path().is_file()
            && entry.path().file_stem().map(|s| s.to_string_lossy())
                == Some(std::borrow::Cow::Borrowed("index"))
        {
            let path = entry.path();
            let mut route_params = RouteParams::from_path(&path);
            if let Some(next) = route_params.next() {
                if let Some(dir) = entry.path().parent() {
                    let file = std::fs::File::create(dir.join("generated.ts"))?;
                    let mut writer = BufWriter::new(file);
                    let _ = writer.write(
                        b"export interface RouteParams extends Record<string, string | undefined> {\n",
                    )?;
                    if let Some(param) = next.strip_prefix("...") {
                        writer.write_fmt(format_args!("  \"{}\"?: string\n", param))?;
                    } else {
                        writer.write_fmt(format_args!("  \"{}\": string\n", next))?;
                    }
                    for param in route_params {
                        if let Some(param) = param.strip_prefix("...") {
                            writer.write_fmt(format_args!("  \"{}\"?: string\n", param))?;
                            continue;
                        }
                        writer.write_fmt(format_args!("  \"{}\": string\n", param))?;
                    }
                    let _ = writer.write(b"}")?;
                    writer.flush()?;
                    *count += 1;
                }
            }
        }
    }
    Ok(())
}
