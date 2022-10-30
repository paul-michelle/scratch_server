use scratch_server as lib;
use std::net::TcpListener;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = lib::ThreadPool::build(4).unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.execute(|| {
            lib::handle_connection(stream);
        });
    }
}
