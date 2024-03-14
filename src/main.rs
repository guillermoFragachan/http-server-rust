use std::{
    io::{Write},
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

    let request_line = request[0].clone();
    let request_parts: Vec<&str> = request_line.splitn(3, ' ').collect();
    let path = request_parts[1];

    let status_code = if path.starts_with("/echo/") {
        200
    } else {
        404
    };

    let body = if status_code == 200 {
        let echo_string = path.split('/').last().unwrap();
        echo_string.to_string()
    } else {
        String::new()
    };

    let response_header = format!("HTTP/1.1 {} OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n", status_code, body.len());
    let response = format!("{}{}", response_header, body);
    stream.write_all(response.as_bytes()).unwrap();
}
