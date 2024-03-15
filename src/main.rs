use std::{
    collections::HashMap,
    env, fs,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
    sync::{Once, OnceLock},
    thread,
};
use anyhow::Context;
static FILE_DIR: OnceLock<String> = OnceLock::new();
fn main() -> anyhow::Result<()> {
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--directory" {
            FILE_DIR
                .set(args.next().context("no directory given!")?)
                .unwrap();
        }
    }
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    let mut threads = Vec::new();
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let handle = thread::spawn(|| handle_connection(stream));
                threads.push(handle);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
    for handle in threads {
        handle.join().unwrap()?;
    }
    Ok(())
}
#[derive(Debug, PartialEq, Eq)]
enum Method {
    Get,
    Post,
}
fn handle_connection(mut stream: TcpStream) -> anyhow::Result<()> {
    let reader = BufReader::new(&mut stream);
    println!("accepted new connection");
    let lines = reader
        .lines()
        .map(|l| l.unwrap())
        .take_while(|line| !line.is_empty())
        .collect::<Vec<String>>();
    assert!(lines.len() > 0, "no start line given!");
    let start_line = &lines[0];
    let (method, path) = parse_start_line(start_line)?;
    let headers = parse_headers(&lines);
    if method == Method::Get {
        if path.starts_with("/echo/") {
            let echo_text = &path[6..];
            return send_plaintext(&mut stream, echo_text);
        }
        if path == "/user-agent" {
            let user_agent = headers
                .get("User-Agent")
                .context("User-Agent header was not set")?;
            return send_plaintext(&mut stream, user_agent);
        }
        if path.starts_with("/files/") {
            if let Some(file_dir) = FILE_DIR.get() {
                let file_path = PathBuf::from(file_dir).join(&path[7..]);
                return match fs::read(&file_path) {
                    Ok(bytes) => send_bytes(&mut stream, &bytes),
                    Err(_e) => {
                        write!(stream, "HTTP/1.1 404 Not Found\r\n\r\n").context("write failed")
                    }
                };
            }
        }
        if path == "/" {
            return write!(stream, "HTTP/1.1 200 OK\r\n\r\n").context("write failed");
        }
    }
    write!(stream, "HTTP/1.1 404 Not Found\r\n\r\n").context("write failed")
}

fn parse_start_line<'a>(start_line: &'a str) -> anyhow::Result<(Method, &'a str)> {
    let mut parts = start_line.split_ascii_whitespace();
    let method = parts.next().context("method must exist")?;
    let path = parts.next().context("path must exist")?;
    let method = match method {
        "GET" => Method::Get,
        "POST" => Method::Post,
        other => anyhow::bail!("HTTP method {other} is not supported"),
    };
    Ok((method, path))
}

fn parse_headers<'a>(lines: &'a [String]) -> HashMap<&'a str, &'a str> {
    lines
        .iter()
        .skip(1)
        .filter_map(|line| line.split_once(':'))
        .map(|(k, v)| (k.trim(), v.trim()))
        .collect()
}

fn send_echo_response(stream: &mut TcpStream, path: &str) -> anyhow::Result<()> {
    send_plaintext(stream, &path[6..])
}

fn send_plaintext(stream: &mut TcpStream, text: &str) -> anyhow::Result<()> {
    write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
        text.len(),
        text
    )
    .context("failed to send plaintext")
}

fn send_user_agent(stream: &mut TcpStream, headers: &HashMap<&str, &str>) -> anyhow::Result<()> {
    let user_agent = *headers
        .get("User-Agent")
        .context("no User-Agent header was sent!")?;
    send_plaintext(stream, user_agent)
}

fn send_bytes(stream: &mut TcpStream, bytes: &[u8]) -> anyhow::Result<()> {
    write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n",
        bytes.len(),
    )
    .context("failed to send bytes")?;
    stream.write_all(bytes).context("failed to send bytes")
}

