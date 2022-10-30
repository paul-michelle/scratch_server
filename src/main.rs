use scratch_server as lib;
use std::net::TcpListener;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = lib::ThreadPool::build(4).unwrap();

    match std::env::args().nth(1) {
        Some(requests_count) => {
            let reqs_to_take: usize = requests_count.parse().unwrap();
            for stream in listener.incoming().take(reqs_to_take) {
                let stream = stream.unwrap();
                pool.execute(|| {
                    lib::handle_connection(stream);
                });
            }
            println!("Shutting down.")
        }
        None => {
            for stream in listener.incoming() {
                let stream = stream.unwrap();
                pool.execute(|| {
                    lib::handle_connection(stream);
                });
            }
        }
    }
}
