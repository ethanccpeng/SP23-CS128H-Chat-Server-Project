use std::net::TcpListener;
use std::net::TcpStream;
use std::io::prelude::*;
use std::fs;
use std::thread;
use std::time::Duration;
use server::ThreadPool;

fn main() {
    // bind to port 7878
    let listener: TcpListener = 
        TcpListener::bind("127.0.0.1:7878").unwrap();
    // create a thread pool
    let pool = ThreadPool::new(4);
    // iterate over incoming connections
    for stream in listener.incoming().take(200) { // 200 max connections
        let stream = stream.unwrap();
        pool.execute(|| {
            handle_connection(stream);
        });
    }       
}

fn handle_connection(mut stream: TcpStream) {
    // read request into buffer
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    // set response based on request
    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    let (status_line, filename) =
        if buffer.starts_with(get) {
            // normal request
            ("HTTP/1.1 200 OK", "index2.html")
        } else if buffer.starts_with(sleep) {
            // simulate slow request
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "index2.html")
        } else {
            // 404 error
            ("HTTP/1.1 404 NOT FOUND", "404.html")
    };
    // read file contents into response
    let contents = fs::read_to_string(filename).unwrap();
    // write response to stream
    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );
    stream.write(response.as_bytes()).unwrap();
    // flush stream
    stream.flush().unwrap();
}