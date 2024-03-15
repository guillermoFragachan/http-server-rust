use std::collections::HashMap;
// Uncomment this block to pass the first stage
use std::io::{self, BufRead};
use std::net::TcpStream;
use std::{io::Write, net::TcpListener};
static CRLF: &str = "\r\n";
fn connect(mut _stream: TcpStream) {
    println!("accepted new connection");
    let reader = io::BufReader::new(&_stream);
    let lines: Vec<_> = reader
        .lines()
        .map(|l| l.unwrap())
        .take_while(|l| l != "")
        .collect();
    let (_method, path, _version) = {
        let parts: Vec<_> = lines[0].split_whitespace().collect();
        (parts[0], parts[1], parts[2])
    };
    let mut headers: HashMap<String, String> = HashMap::new();
    for line in lines.iter().skip(1) {
        let parts: Vec<_> = line.splitn(2, ": ").collect();
        headers.insert(parts[0].to_string(), parts[1].to_string());
    }
    match path {
        "/" => {
            let resp_status_line = "HTTP/1.1 200 OK\r\n";
            _stream.write(resp_status_line.as_bytes()).unwrap();
            _stream.write(CRLF.as_bytes()).unwrap();
        }
        // starts with /echo
        _ if path.starts_with("/echo/") => {
            let resp_status_line = "HTTP/1.1 200 OK\r\n";
            _stream.write(resp_status_line.as_bytes()).unwrap();
            let echo = path.splitn(2, "/echo/").collect::<Vec<&str>>()[1];
            _stream
                .write("Content-Type: text/plain\r\n".as_bytes())
                .unwrap();
            _stream
                .write(format!("Content-Length: {}\r\n", echo.len()).as_bytes())
                .unwrap();
            _stream.write(CRLF.as_bytes()).unwrap();
            _stream.write(echo.as_bytes()).unwrap();
        }
        _ if path.starts_with("/user-agent") => {
            let resp_status_line = "HTTP/1.1 200 OK\r\n";
            _stream.write(resp_status_line.as_bytes()).unwrap();
            _stream
                .write("Content-Type: text/plain\r\n".as_bytes())
                .unwrap();
            let user_agent = headers.get("User-Agent").unwrap();
            _stream
                .write(format!("Content-Length: {}\r\n", user_agent.len()).as_bytes())
                .unwrap();
            _stream.write(CRLF.as_bytes()).unwrap();
            _stream.write(user_agent.as_bytes()).unwrap();
        }
        _ => {
            let resp_status_line = "HTTP/1.1 404 Not Found\r\n";
            _stream.write(resp_status_line.as_bytes()).unwrap();
            _stream.write(CRLF.as_bytes()).unwrap();
        }
    }
}
fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");
    // Uncomment this block to pass the first stage
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => connect(_stream),
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}