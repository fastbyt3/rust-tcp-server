#![allow(dead_code)]
use std::str;
use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    str::FromStr,
    thread,
};

#[derive(Debug, PartialEq)]
enum HttpMethod {
    GET,
    POST,
}

impl FromStr for HttpMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(HttpMethod::GET),
            "POST" => Ok(HttpMethod::POST),
            _ => Err(format!("Unexpected HTTP method -> {}", s)),
        }
    }
}

struct RequestStartLine {
    method: HttpMethod,
    path: String,
    version: String,
}

impl RequestStartLine {
    fn construct(s: &str) -> RequestStartLine {
        let mut start_line_parts = s.split_whitespace();
        let method = start_line_parts
            .next()
            .unwrap()
            .parse::<HttpMethod>()
            .unwrap();
        let path = start_line_parts.next().unwrap().to_string();
        let version = start_line_parts.next().unwrap().to_string();

        RequestStartLine {
            method,
            path,
            version,
        }
    }
}

struct Request {
    start_line: RequestStartLine,
    headers: HashMap<String, String>,
    body: String,
}

impl FromStr for Request {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.split("\r\n");

        let start_line = RequestStartLine::construct(lines.next().unwrap());

        let mut headers: HashMap<String, String> = HashMap::new();

        for line in lines.by_ref() {
            if line.len() == 0 {
                break;
            }
            if let Some((key, value)) = line.split_once(':') {
                headers.insert(key.to_string(), value.trim().to_string());
            }
        }

        let mut other_parts: Vec<&str> = vec![];
        for remaining_lines in lines {
            other_parts.push(remaining_lines);
        }
        let body = other_parts.join("");
        println!("Body: {}", body);

        Ok(Request {
            start_line,
            headers,
            body,
        })
    }
}

fn handle_connection(mut stream: TcpStream) {
    let ok_200 = "HTTP/1.1 200 OK\r\n";
    let ok_201 = "HTTP/1.1 201 OK\r\n";
    let not_found_404 = "HTTP/1.1 404 NOT FOUND\r\n\r\n";

    let mut buf = [0; 1024];
    let data_recv_bytes = stream
        .read(&mut buf)
        .expect("Error reading stream into a buffer");
    let request = str::from_utf8(&buf[..data_recv_bytes])
        .expect("Unable to parse request as &str")
        .parse::<Request>()
        .expect("Unable to parse Request");

    let path: String = request
        .start_line
        .path
        .split_inclusive('/')
        .take(2)
        .collect();
    let req = (request.start_line.method, path.as_str());
    let response: String = match req {
        (HttpMethod::GET, "/") => "HTTP/1.1 200 OK\r\n\r\n".to_string(),
        (HttpMethod::GET, "/user-agent") => {
            let ua = request
                .headers
                .get("User-Agent")
                .expect("Couldnt find User-Agent key in headers map");
            format!(
                "{}Content-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                ok_200,
                ua.len(),
                ua
            )
        }
        (HttpMethod::GET, "/echo/") => {
            let echo = request.start_line.path.strip_prefix("/echo/").unwrap();
            format!(
                "{}Content-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                ok_200,
                echo.len(),
                echo
            )
        }
        (HttpMethod::GET, "/files/") => {
            let file_name: String = request.start_line.path
                .strip_prefix("/files/")
                .expect("Couldnt get filename")
                .to_string();
            let directory: String = std::env::args()
                .nth(2)
                .expect("Didnt receive absoulte path")
                .to_string();
            let file_path = format!("{}/{}", directory, file_name);

            if let Ok(content) = std::fs::read_to_string(file_path) {
                let x = format!(
                    "{}Content-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}",
                    ok_200,
                    content.len(),
                    content
                );
                x
            } else {
                format!("{}", not_found_404)
            }
        }
        (HttpMethod::POST, "/files/") => {
            let file_name: String = request.start_line.path
                .strip_prefix("/files/")
                .expect("Couldnt get filename")
                .to_string();
            let directory: String = std::env::args()
                .nth(2)
                .expect("Didnt receive absoulte path")
                .to_string();
            let file_path = format!("{}/{}", directory, file_name);

            match std::fs::write(file_path, request.body) {
                Ok(_) => format!("{}\r\n\r\n", ok_201),
                Err(_) => format!("{}", not_found_404)
            }
        }
        (_, _) => format!("{}", not_found_404),
    };

    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn main() {
    println!("Logs from your program will appear here!");

    let listener =
        TcpListener::bind("127.0.0.1:4221").expect("Couldn't start a TCP Listener at port 4221");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| {
                    println!("accepted new connection");
                    handle_connection(stream);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
