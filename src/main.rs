mod html_writer;
mod imports;
mod jobs;
mod route_params;
mod threadpool;
mod types;
mod utils;
mod yaml;

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use threadpool::Job;
use types::{Config, Content};

use crate::threadpool::ThreadPool;

fn main() {
    let input = PathBuf::from("examples/blog/src/content");
    let output = PathBuf::from("examples/blog/src/content-generated");
    let routes = PathBuf::from("examples/blog/src/routes");
    if let Err(e) = std::fs::remove_dir_all(&output) {
        println!("Remove Dir: {}", e);
    }
    if let Err(e) = std::fs::create_dir_all(&output.join("files")) {
        println!("Create Dir: {}", e)
    }
    let size = std::fs::read_dir(&input).unwrap().count();
    let config = Arc::new(Config::new(input, output, routes));
    let pool = ThreadPool::new(8);
    let content = Arc::new(process_content(size, &pool, config.clone()));

    pool.execute(Job::ProcessCollections(content.clone(), config.clone()));
    pool.execute(Job::ProcessTaxonomies(content.clone(), config.clone()));
    pool.execute(Job::ProcessMarkdown(content.clone(), config.clone()));
    pool.execute(Job::ProcessMDX(content.clone(), config.clone()));
    pool.execute(Job::GenerateRouteParams(config.clone()));
    if !content.is_empty() {
        pool.execute(Job::WriteHelpers(config))
    }
    println!("{} content files", content.len())
}

fn process_content(size: usize, pool: &ThreadPool, config: Arc<Config>) -> Content {
    let mut content = Content::with_capacity(size);
    process_content_rec(&config.input, &mut content, pool, config.clone());
    content
}

fn process_content_rec(curr: &Path, content: &mut Content, pool: &ThreadPool, config: Arc<Config>) {
    if let Ok(dir) = std::fs::read_dir(curr) {
        for entry in dir.filter_map(|e| e.ok()) {
            if entry.path().is_dir() && entry.path() != config.output {
                process_content_rec(&entry.path(), content, pool, config.clone());
            }
            if entry.path().is_file() {
                match std::fs::read_to_string(entry.path()) {
                    Ok(file) => content.push_file(entry.path(), &file),
                    Err(e) => println!("{}", e),
                }
            }
        }
    }
}
