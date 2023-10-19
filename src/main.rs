#![allow(dead_code)]
use std::{net::{TcpListener, TcpStream}, io::{Write, Read}, iter::Map, str::FromStr, collections::HashMap};
use std::str;

enum HttpMethod {
    GET
}

impl FromStr for HttpMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(HttpMethod::GET),
            _ => Err(format!("Unexpected HTTP method -> {}", s))
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
        let method = start_line_parts.next()
            .unwrap()
            .parse::<HttpMethod>()
            .unwrap();
        let path = start_line_parts.next().unwrap().to_string();
        let version = start_line_parts.next().unwrap().to_string();

        RequestStartLine { method, path, version }
    }
}

struct Request {
    start_line: RequestStartLine,
    headers: HashMap<String, String>,
}

impl FromStr for Request {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.split("\r\n");

        let start_line = RequestStartLine::construct(lines.next().unwrap());

        let mut headers: HashMap<String, String> = HashMap::new();
        for line in lines {
            if line.len() == 0 {
                break;
            }
            if let Some((key, value)) = line.split_once(':') {
                headers.insert(key.to_string(), value.trim().to_string());
            }
        }

        Ok(Request { start_line, headers })
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buf = [0; 1024];
    stream.read(&mut buf).expect("Error reading stream into a buffer");
    let request = str::from_utf8(&buf).expect("Unable to parse request as &str").parse::<Request>().expect("Unable to parse Request");

    let response: String = match request.start_line.path.as_str() {
        "/" => "HTTP/1.1 200 OK\r\n\r\n".to_string(),
        _ if request.start_line.path.starts_with("/echo/") => {
            let echo = request.start_line.path.strip_prefix("/echo/").unwrap();
            println!("{}", echo);
            format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", echo.len(), echo)
        },
        "/user-agent" => {
            let ua = request.headers.get("User-Agent").expect("Couldnt find User-Agent key in headers map");
            format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", ua.len(), ua)
        }
        _ => "HTTP/1.1 404 NOT FOUND\r\n\r\n".to_string()
    };

    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn main() {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").expect("Couldn't start a TCP Listener at port 4221");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("accepted new connection");
                handle_connection(stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
