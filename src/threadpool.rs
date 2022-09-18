use std::{
    sync::{
        mpsc::{Receiver, Sender},
        Arc, Mutex,
    },
    thread::JoinHandle,
    time::Instant,
};

use crate::types::{Config, Content};

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
    #[allow(dead_code)]
    id: usize,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Self {
        let thread = std::thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv().unwrap();
            // println!("Worker {} got a job; executing.", id);
            match job {
                Job::WriteHelpers(config) => {
                    if let Err(e) = crate::jobs::write_type_helpers::generate(config) {
                        println!("Helpers {}", e)
                    }
                }
                Job::GenerateRouteParams(config) => {
                    if let Err(e) = crate::jobs::generate_route_params::generate(&config.routes) {
                        println!("Params {}", e)
                    }
                }
                Job::ProcessCollections(content, config) => {
                    if crate::jobs::write_collections::process_all(content, config).is_err() {
                        println!("Yaml Error error");
                    }
                }
                Job::ProcessTaxonomies(content, config) => {
                    crate::jobs::write_taxonomies::process_all(content, config)
                }
                Job::ProcessMarkdown(content, config) => {
                    if let Err(e) = crate::jobs::process_markdown::process_all(content, config) {
                        println!("Markdown {}", e)
                    }
                }
                Job::ProcessMDX(content, config) => {
                    if let Err(e) = crate::jobs::process_mdx::process_all(content, config) {
                        println!("Markdown {}", e)
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
    GenerateRouteParams(Arc<Config>),
    WriteHelpers(Arc<Config>),
    ProcessCollections(Arc<Content>, Arc<Config>),
    ProcessTaxonomies(Arc<Content>, Arc<Config>),
    ProcessMarkdown(Arc<Content>, Arc<Config>),
    ProcessMDX(Arc<Content>, Arc<Config>),
    Terminate,
}
