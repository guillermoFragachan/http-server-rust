use std::collections::HashMap;
// Uncomment this block to pass the first stage
use std::io::{self, BufRead};
use std::net::TcpStream;
use std::thread;
use std::{io::Write, net::TcpListener};
use std::fs::File;
use std::io::Read;
use std::fs;
use std::env;

static CRLF: &str = "\r\n";

fn print_directory_contents(directory_path: &str) {
    match fs::read_dir(directory_path) {
        Ok(files) => {
            for file in files {
                match file {
                    Ok(file) => {
                        let file_name = file.file_name().into_string().unwrap_or_else(|_| String::from("<invalid encoding>"));
                        println!("{}", file_name);
                    }
                    Err(e) => {
                        println!("Error reading file: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            println!("Error reading directory: {}", e);
        }
    }
}
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
        _ if path.starts_with("/files/") =>{
            let resp_status_line = "HTTP/1.1 200 OK\r\n";
            _stream.write(resp_status_line.as_bytes()).unwrap();
            let content: Vec<&str> = path.split("files/").collect();


            let filename = content[1];
            let directory = env::args().nth(2).unwrap();
            let file_name = format!("{directory}/{filename}");
        
            // print_directory_contents("./");
        
            // Check if the file exists in the local directory
            let file_path = format!("./{}", file_name);
            if let Ok(mut file) = File::open(file_path) {
                // Set the content type based on the file extension (optional)
                let content_type = match file_name.rsplit('.').next() {
                    Some("txt") => "text/plain",
                    Some("html") => "text/html",
                    Some("css") => "text/css",
                    Some("js") => "application/javascript",
                    Some(_) => "application/octet-stream",
                    None => "application/octet-stream",
                };
                _stream.write(format!("Content-Type: {}\r\n", content_type).as_bytes()).unwrap();
        
                // Read the file content and send it in the response
                let mut file_content = Vec::new();
                file.read_to_end(&mut file_content).unwrap();
                _stream.write(format!("Content-Length: {}\r\n", file_content.len()).as_bytes()).unwrap();
                _stream.write(CRLF.as_bytes()).unwrap();
                _stream.write(&file_content).unwrap();
            } else {
                // File not found, send a 404 response
                let resp_status_line = "HTTP/1.1 404 Not Found\r\n";
                _stream.write(resp_status_line.as_bytes()).unwrap();
                _stream.write(CRLF.as_bytes()).unwrap();
            }
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
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    let mut threads = Vec::new();
    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                let handle = thread::spawn(|| {
                    connect(_stream);
                });
                threads.push(handle);
            },
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }

    
}
