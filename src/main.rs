use std::{net::{TcpListener, TcpStream}, io::{Write, Read}};
use std::str;

fn handle_connection(mut stream: TcpStream) {
    let mut buf = [0; 1024];
    stream.read(&mut buf).unwrap();
    let request = str::from_utf8(&buf).expect("Unable to parse request as &str");
    println!("{}", request);
    let path = request.split_whitespace().nth(1).expect("PATH was not found at specified position");
    println!("{}", path);

    let response: String = match path {
        "/" => "HTTP/1.1 200 OK\r\n\r\n".to_string(),
        _ if path.starts_with("/echo/") => {
            let echo = path.strip_prefix("/echo/").expect("No content to echo.... nothing beyond /echo/");
            println!("{}", echo);
            format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", echo.len(), echo)
        },
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
