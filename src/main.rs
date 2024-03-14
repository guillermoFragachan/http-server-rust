use std::{
    io::{Read, Write},
    io::{BufRead, BufReader},
    net::{TcpListener, TcpStream},
};
fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                handle_connection(_stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();
    if request[0] == "GET / HTTP/1.1" {
        let response = b"HTTP/1.1 200 OK\r\n\r\n";
        stream.write_all(response).unwrap();
    } else {
        let response = b"HTTP/1.1 404 NOT FOUND\r\n\r\n";
        stream.write_all(response).unwrap();
    }
}
