use std::net::TcpListener;

use simple_smtp::{handle_connection, thread_pool::ThreadPool};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);


    for stream in listener.incoming() {
        let stream = stream.unwrap();
        println!("Connection established!");

        pool.execute(|| {handle_connection(stream)});
    }
}
