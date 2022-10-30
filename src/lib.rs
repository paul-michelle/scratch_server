#![allow(unused_variables, dead_code)]

use std::{
    collections::HashMap,
    fs,
    io::{BufRead, BufReader, Write},
    net::TcpStream,
};

pub struct ThreadPool;
impl ThreadPool {
    /// Create a new ThreadPool with the `size` being the max number of threads.
    /// The `new` func will panic if the size is 0.
    pub fn new(size: usize) -> Self {
        assert!(size > 0);
        ThreadPool {}
    }
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
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
