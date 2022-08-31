mod jobs;
mod route_params;
mod threadpool;
mod types;
mod utils;

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use threadpool::Job;
use types::{Config, FrontMatter, GeneratedData};
use utils::get_content_ranges;

use crate::threadpool::ThreadPool;

fn main() {
    let input = PathBuf::from("examples/blog/src/content");
    let output = PathBuf::from("examples/blog/src/content-generated");
    let config = Arc::new(Config::new(input, output));
    let mut pool = ThreadPool::new(8);

    pool.execute(Job::GenerateRouteParams(PathBuf::from(
        "examples/blog/src/routes",
    )));
    let generated = Arc::new(process_content_dir(&mut pool, config.clone()));
    pool.execute(Job::GenerateTaxonomies(config.clone(), generated.clone()));
    pool.execute(Job::GenerateCollections(config.clone(), generated.clone()));
    if !generated.output_paths.is_empty() {
        pool.execute(Job::WriteHelpers(config))
    }
}

fn process_content_dir(pool: &mut ThreadPool, config: Arc<Config>) -> GeneratedData {
    let mut gen = GeneratedData::default();
    process_content_rec(&config.input, pool, &mut gen, config.clone());
    gen
}

fn process_content_rec(
    curr: &Path,
    pool: &mut ThreadPool,
    gen: &mut GeneratedData,
    config: Arc<Config>,
) {
    if let Ok(dir) = std::fs::read_dir(curr) {
        for entry in dir.filter_map(|e| e.ok()) {
            if entry.path().is_dir() {
                process_content_rec(&entry.path(), pool, gen, config.clone());
            }
            if entry.path().is_file() {
                if let Ok(file) = std::fs::read_to_string(entry.path()) {
                    let path = entry.path();
                    let id = gen.output_paths.len();
                    let ranges = get_content_ranges(file.as_bytes());
                    let frontmatter: FrontMatter = serde_yaml::from_str(
                        &file[ranges.frontmatter.start..ranges.frontmatter.end],
                    )
                    .unwrap();
                    let rel = path.strip_prefix(&config.input).unwrap();
                    let dir = rel.parent().unwrap();
                    for tag in frontmatter.tags.iter() {
                        if let Some(vec) = gen.collections.get_mut(tag) {
                            vec.push(id);
                        } else {
                            gen.collections.insert(tag.to_owned(), vec![id]);
                        }
                    }
                    if let Some(vec) = gen.collections.get_mut("all") {
                        vec.push(id);
                    } else {
                        gen.collections.insert("all".to_owned(), vec![id]);
                    }
                    for segment in dir {
                        let key = segment.to_str().unwrap();
                        if let Some(vec) = gen.taxonomies.get_mut(key) {
                            vec.push(id)
                        } else {
                            gen.taxonomies.insert(key.to_owned(), vec![id]);
                        }
                    }
                    gen.output_paths.push(rel.to_path_buf());

                    pool.execute(Job::ProcessMD(ranges, file, entry.path(), config.clone()))
                }
            }
        }
    }
}
