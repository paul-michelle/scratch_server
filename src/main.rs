use scratch_server as lib;
use std::net::TcpListener;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        lib::handle_connection(stream.unwrap());
    }
}
