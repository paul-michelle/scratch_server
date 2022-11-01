use scratch_server as lib;
use std::net::TcpListener;

struct Config {
    server_addr: String,
    requests_to_serve: Option<usize>,
}

impl Config {
    fn new(mut args: impl Iterator<Item = String>) -> Self {
        args.next();

        let server_addr = match args.next() {
            Some(arg) => arg,
            None => String::from("127.0.0.1:7878"),
        };
        let requests_to_serve: Option<usize> = match args.next() {
            Some(arg) => match arg.parse::<usize>() {
                Ok(count) => Some(count),
                Err(_) => None,
            },
            None => None,
        };
        Self {
            server_addr,
            requests_to_serve,
        }
    }
}

fn main() {
    let config = Config::new(std::env::args());
    let listener = TcpListener::bind(config.server_addr).unwrap();
    let pool = lib::ThreadPool::build(4).unwrap();

    if let Some(limit) = config.requests_to_serve {
        println!("Going to serve {limit} request(s) at most.");
        for stream in listener.incoming().take(limit) {
            let stream = stream.unwrap();
            pool.execute(|| {
                lib::handle_connection(stream);
            });
        }
        println!("Shutting down.");
        return;
    }

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.execute(|| {
            lib::handle_connection(stream);
        });
    }
}
