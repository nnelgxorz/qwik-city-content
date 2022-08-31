#![allow(dead_code)]
use std::{
    path::PathBuf,
    sync::{
        mpsc::{Receiver, Sender},
        Arc, Mutex,
    },
    thread::JoinHandle,
    time::Instant,
};

use crate::{
    types::{Config, GeneratedData},
    utils::ContentRanges,
};

pub struct ThreadPool {
    start: Instant,
    sender: Sender<Job>,
    workers: Vec<Worker>,
}

impl ThreadPool {
    pub fn new(size: usize) -> Self {
        assert!(size > 0);
        let mut workers = Vec::with_capacity(size);
        let (sender, receiver) = std::sync::mpsc::channel::<Job>();
        let receiver = Arc::new(Mutex::new(receiver));
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)))
        }
        Self {
            start: Instant::now(),
            sender,
            workers,
        }
    }
    pub fn execute(&self, job: Job) {
        self.sender.send(job).unwrap()
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for _ in &mut self.workers {
            self.sender.send(Job::Terminate).unwrap();
        }
        for worker in &mut self.workers {
            // println!("Shutting down worker {}", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
        println!(
            "Finished in {:.3}ms",
            self.start.elapsed().as_nanos() as f32 / 1000000.0
        )
    }
}

#[derive(Debug)]
struct Worker {
    id: usize,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Self {
        let thread = std::thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv().unwrap();
            // println!("Worker {} got a job; executing.", id);
            match job {
                Job::ProcessMD(ranges, content, path, config) => {
                    let _ = crate::jobs::process_markdown::generate(ranges, content, path, config);
                }
                Job::ProcessMDX(_, _, path, _) => {
                    println!(
                        "Skipping {} .mdx is not currently supported",
                        path.display()
                    );
                }
                Job::GenerateTaxonomies(config, gen) => {
                    if let Err(e) = crate::jobs::write_taxonomies::generate(config, gen) {
                        println!("{}", e)
                    }
                }
                Job::GenerateCollections(config, gen) => {
                    if let Err(e) = crate::jobs::write_collections::generate(config, gen) {
                        println!("{}", e)
                    }
                }
                Job::WriteHelpers(config) => {
                    if let Err(e) = crate::jobs::write_type_helpers::generate(config) {
                        println!("{}", e)
                    }
                }
                Job::GenerateRouteParams(routes) => {
                    if let Err(e) = crate::jobs::generate_route_params::generate(routes) {
                        println!("{}", e)
                    }
                }
                Job::Terminate => {
                    break;
                }
            }
        });
        Self {
            id,
            thread: Some(thread),
        }
    }
}

pub enum Job {
    ProcessMD(ContentRanges, String, PathBuf, Arc<Config>),
    ProcessMDX(ContentRanges, String, PathBuf, Arc<Config>),
    GenerateTaxonomies(Arc<Config>, Arc<GeneratedData>),
    GenerateCollections(Arc<Config>, Arc<GeneratedData>),
    GenerateRouteParams(PathBuf),
    WriteHelpers(Arc<Config>),
    Terminate,
}
