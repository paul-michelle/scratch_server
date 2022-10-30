use std::{
    collections::HashMap,
    error::Error,
    fmt::Display,
    fs,
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    sync::{mpsc, Arc, Mutex},
    thread::{self, JoinHandle},
};

#[derive(Debug)]
pub struct PoolCreationError;
impl Error for PoolCreationError {}
impl Display for PoolCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Failed to create a ThreadPool. Poolsize should be gte 1."
        )
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker {
    id: u8,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    fn new(id: u8, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Option<Self> {
        let thread_builder = thread::Builder::new();
        let spawned_thread = thread_builder.spawn(move || loop {
            let msg = receiver.lock().unwrap().recv();
            match msg {
                Ok(job) => {
                    println!("Worker {id} got a job! Executing...");
                    job();
                }
                Err(_) => {
                    println!("Worker {id} disconnected from channel. Shutting down...");
                    break;
                }
            }
        });

        match spawned_thread {
            Ok(thread) => Some(Worker {
                id,
                thread: Some(thread),
            }),
            Err(_) => None,
        }
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    pub fn build(size: usize) -> Result<Self, PoolCreationError> {
        if size == 0 {
            return Err(PoolCreationError {});
        }
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);
        for i in 0..size {
            if let Some(worker) = Worker::new(i as u8, Arc::clone(&receiver)) {
                workers.push(worker);
            }
        }
        Ok(ThreadPool {
            workers,
            sender: Some(sender),
        })
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        if let Some(sender) = self.sender.take() {
            println!("Closing the channel's producing end.");
            drop(sender);
        }
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

pub struct RequestMapper {
    mapping: HashMap<&'static str, (&'static str, &'static str)>,
}

impl RequestMapper {
    fn new() -> Self {
        RequestMapper {
            mapping: HashMap::new(),
        }
    }
    pub fn init() -> Self {
        let mut m = Self::new();
        m.mapping
            .insert("GET / HTTP/1.1", ("HTTP/1.1 200 OK", "index.html"));
        m
    }
    pub fn get(&self, key: &str) -> (&'static str, &'static str) {
        let res = self
            .mapping
            .get(key)
            .unwrap_or(&("HTTP/1.1 404 NOT FOUND", "404.html"));
        *res
    }
}

pub struct Renderer;
impl Renderer {
    pub fn template_to_string(template_name: &str) -> String {
        let template_path = format!("static/{}", template_name);
        match fs::read_to_string(template_path) {
            Ok(rendered) => rendered,
            Err(_) => String::from(""),
        }
    }
}

pub fn handle_connection(mut stream: TcpStream) {
    let mapper = RequestMapper::init();

    let buf_reader = BufReader::new(&mut stream);
    let request_line_parse_result = match buf_reader.lines().next() {
        Some(result) => result,
        None => return,
    };
    let request_line = match request_line_parse_result {
        Ok(line) => line,
        Err(_) => return,
    };
    let (status_line, template_name) = mapper.get(&request_line);
    let contents = Renderer::template_to_string(template_name);
    let content_length = contents.len();
    let resp = format!("{status_line}\r\nContent-Length: {content_length}\r\n\r\n{contents}");
    if stream.write_all(resp.as_bytes()).is_ok() {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_mapper() {
        let mapper = RequestMapper::init();
        let (status_line, template_name) = mapper.get("GET / HTTP/1.1");
        assert_eq!(status_line, "HTTP/1.1 200 OK");
        assert_eq!(template_name, "index.html");

        let (status_line, template_name) = mapper.get("GET /ping HTTP/1.1");
        assert_eq!(status_line, "HTTP/1.1 404 NOT FOUND");
        assert_eq!(template_name, "404.html");

        let (status_line, template_name) = mapper.get("VERB / HTTP/1.1");
        assert_eq!(status_line, "HTTP/1.1 404 NOT FOUND");
        assert_eq!(template_name, "404.html");
    }
}
