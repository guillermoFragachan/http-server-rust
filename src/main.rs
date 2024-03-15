
use std::{
    env,
    fs::{self, File},
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf}, thread,
};
use itertools::Itertools;

#[derive(Debug)]
enum ReturnCode {
    OK,
    NotFound,
    Created,
}

impl ReturnCode {
    fn get_message(&self) -> String {
        match self {
            ReturnCode::OK => String::from("HTTP/1.1 200 OK\r\n"),
            ReturnCode::NotFound => String::from("HTTP/1.1 404 Not Found\r\n"),
            ReturnCode::Created => String::from("HTTP/1.1 201 Created\r\n"),
        }
    }
}

#[derive(Debug, PartialEq)]
enum Verb {
    GET,
    POST,
}

impl PartialEq<&str> for Verb {
    fn eq(&self, other: &&str) -> bool {
        match self {
            Verb::GET => *other == "GET",
            Verb::POST => *other == "POST",
        }
    }
}

#[derive(Debug)]
struct Response {
    code: ReturnCode,
    text: String,
}

impl Response {
    fn new(code: ReturnCode, text: String) -> Self {
        Response { code, text }
    }

    fn new_partial(code: ReturnCode) -> Self {
        Response {
            code,
            text: String::new(),
        }
    }

    fn get_message(&self) -> String {
        let header: String = self.code.get_message();
        let content_type = "text/plain";
        let content_length = self.text.len();
        let content: String = self.text.clone();
        if !self.text.is_empty() {
            format!("{header}Content-Type: {content_type}\r\nContent-length: {content_length}\r\n\r\n{content}")
        } else {
            format!("{header}\r\n")
        }
    }
}

fn read_stream(stream: &TcpStream) -> Vec<String> {
    let reader: BufReader<&TcpStream> = BufReader::new(stream);
    let mut lines: Vec<String> = Vec::new();
    for line in reader.lines() {
        match line {
            Ok(l) => {
                if l.is_empty() {
                    // No more data on the buffer
                    break;
                }
                lines.push(l)
            }
            _ => {
                break;
            }
        }
    }
    lines
}

#[derive(Debug)]
struct Request {
    path: String,
    verb: Verb,
    body: String,
    content_type: String,
    user_agent: String,
    content_length: u32,
}

impl Request {
    fn new(
        path: String,
        verb: Verb,
        body: String,
        content_type: String,
        user_agent: String,
        content_length: u32,
    ) -> Self {
        Request {
            path,
            verb,
            body,
            content_type,
            user_agent,
            content_length,
        }
    }
}

fn handle_request(mut stream: &TcpStream) {
    let data: Vec<String> = read_stream(stream);
    let path: &str = data.get(0).unwrap().split_whitespace().nth(1).unwrap();
    if path == "/" {
        let res: Response = Response::new_partial(ReturnCode::OK);
        stream.write_all(res.get_message().as_bytes()).unwrap();
    } else if path.starts_with("/echo") {
        let parsed_fields: String = path
            .split_inclusive("/")
            .collect::<Vec<&str>>()
            .get(2..)
            .unwrap()
            .join("");
        let resp: Response = Response::new(ReturnCode::OK, parsed_fields);
        stream.write_all(resp.get_message().as_bytes()).unwrap();
    } else if path.starts_with("/user-agent") {
        let mut user_agent_it = data.get(2).unwrap().split_whitespace();
        let user = user_agent_it.nth(1).unwrap();
        let resp = Response::new(ReturnCode::OK, String::from(user));
        stream.write_all(resp.get_message().as_bytes()).unwrap();
    } else if path.starts_with("/files") {
        let args: Vec<String> = env::args().collect();
        let directory = args.last().unwrap();
        let filename: &str = data
            .get(0)
            .unwrap()
            .split_whitespace()
            .nth(1)
            .unwrap()
            .split("/")
            .last()
            .unwrap();
        if let Ok(mut file) = File::open(format!("{directory}{filename}")) {
            let mut content = vec![];
            let len = file.read_to_end(&mut content).unwrap();
            stream.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {len}\r\n\r\n").as_bytes()).unwrap();
            stream.write_all(&content).unwrap();
        } else {
            //Not Found
            let res = Response::new_partial(ReturnCode::NotFound);
            stream.write_all(res.get_message().as_bytes()).unwrap();
        }
    } else {
        let res = Response::new_partial(ReturnCode::NotFound);
        stream.write_all(res.get_message().as_bytes()).unwrap();
    }
}

// fn read_stream(stream: &TcpStream) -> Result<Request, String> {
//     let mut reader: BufReader<&TcpStream> = BufReader::new(stream);
//     let mut line: String = String::new();
//     _ = reader.read_line(&mut line).unwrap();
//     let mut hm: HashMap<String, String> = HashMap::new();
//     // Parse path
//     let mut verb: Verb = Verb::GET;
//     let mut path: String = String::new();
//     if let Some((v, p, _)) = line.split_whitespace().collect_tuple() {
//         match v {
//             "POST" => verb = Verb::POST,
//             "GET" => verb = Verb::GET,
//             _ => {}
//         };
//         path = p.to_string();
//     }
//     loop {
//         line = String::new();
//         _ = reader
//             .read_line(&mut line)
//             .map_err(|_| "Error reading line")?;
//         if line == "\r\n" {
//             break;
//         }
//         if let Some((key, value)) = line.trim().split(": ").collect_tuple() {
//             hm.insert(key.to_string(), value.to_string());
//         }
//     }
//     let content_lengt_value = hm.get(&"Content-Length".to_string());
//     let content_length: usize = match content_lengt_value {
//         Some(content_length_s) => content_length_s
//             .parse()
//             .map_err(|_| "Error parsing content length")?,
//         None => 0,
//     };
//     let mut buffer = vec![0; content_length];
//     reader
//         .read_exact(&mut buffer)
//         .map_err(|_| "error reading body")?;
//     let body = buffer;
//     let body_str = String::from_utf8_lossy(&body).to_string(); // Clone the body_str
//     let content_type = if let Some(v) = hm.get("Content-Type") {
//         v
//     } else {
//         ""
//     };
//     let content_length = if let Some(v) = hm.get("Content-Length") {
//         v.parse::<u32>().unwrap()
//     } else {
//         let res = Response::new_partial(ReturnCode::NotFound);
//         stream.write_all(res.get_message().as_bytes()).unwrap();
//         0
//     };
//     let user_agent = if let Some(v) = hm.get("User-Agent") {
//         v
//     } else {
//         ""
//     };
//     let request: Request = Request::new(
//         path,
//         verb,
//         body_str, // Use the cloned body_str
//         content_type.to_string(),
//         user_agent.to_string(),
//         content_length,
//     );
//     Ok(request)
// }

fn write_file(filepath: &Path, content: &str) -> std::io::Result<()> {
    let dir = filepath.parent().unwrap();
    fs::create_dir_all(dir)?;
    fs::write(filepath, content)?;
    Ok(())
}

fn post_file_content(filepath: &str, dir: &str, content: &str) -> String {
    let mut buf = PathBuf::new();
    buf.push(dir);
    buf.push(filepath);
    let p = buf.as_path();
    if write_file(p, content).is_err() {
        let status_line = "HTTP/1.1 404 Not Found";
        return format!("{status_line}\r\n\r\n");
    }
    let status_line = "HTTP/1.1 201 CREATED";
    let content_type = "application/octet-stream";
    format!("{status_line}\r\nContent-Type: {content_type}\r\n\r\n")
}

// fn handle_request(mut stream: &TcpStream) {
//     let data = read_stream(stream);
//     match data {
//         Err(e) => {
//             panic!("Failed to parse the request: {}", e)
//         },
//         Ok(req) => {
//             if req.path.starts_with("/echo") {
//                 let parsed_fields: String = req
//                     .path
//                     .split_inclusive("/")
//                     .collect::<Vec<&str>>()
//                     .get(2..)
//                     .unwrap()
//                     .join("");
//                 let resp: Response = Response::new(ReturnCode::OK, parsed_fields);
//                 stream.write_all(resp.get_message().as_bytes()).unwrap();
//             } else if req.path.starts_with("/user-agent") {
//                 let resp = Response::new(ReturnCode::OK, req.user_agent);
//                 stream.write_all(resp.get_message().as_bytes()).unwrap();
//             } else if req.path.starts_with("/files") && req.verb == Verb::GET {
//                 let args: Vec<String> = env::args().collect();
//                 let directory = args.last().unwrap();
//                 let filename: &str = req.path.split("/").last().unwrap();
//                 if let Ok(mut file) = File::open(format!("{directory}{filename}")) {
//                     let mut content = vec![];
//                     let len = file.read_to_end(&mut content).unwrap();
//                     stream.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {len}\r\n\r\n").as_bytes()).unwrap();
//                     stream.write_all(&content).unwrap();
//                 } else {
//                     //Not Found
//                     let res = Response::new_partial(ReturnCode::NotFound);
//                     stream.write_all(res.get_message().as_bytes()).unwrap();
//                 }
//             } else if req.path.starts_with("/files") && req.verb == Verb::POST {
//                 let args: Vec<String> = env::args().collect();
//                 let directory: &String = args.last().unwrap();
//                 let filename: &str = req.path.split("/").last().unwrap();
//                 let resp = post_file_content(filename, &directory, &req.body);
//                 stream.write_all(resp.as_bytes()).unwrap();
//             } else if req.path == "/" {
//                 let res: Response = Response::new_partial(ReturnCode::OK);
//                 stream.write_all(res.get_message().as_bytes()).unwrap();
//             } else {
//                 let res = Response::new_partial(ReturnCode::NotFound);
//                 stream.write_all(res.get_message().as_bytes()).unwrap();
//             }
//         }
//     }
// }

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                println!("Accepted new connection...");
                thread::spawn(move || handle_request(&_stream));
                //handle_request(&_stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}