use std::{net::{TcpListener, TcpStream}, io::{Write, Read}};
use std::str;

fn handle_connection(mut stream: TcpStream) {
    let mut buf = [0; 1024];
    stream.read(&mut buf).unwrap();
    let request = str::from_utf8(&buf).expect("Unable to parse request as &str");
    println!("{}", request);
    let path = request.split_whitespace().nth(1).expect("PATH was not found at specified position");
    println!("{}", path);

    let response: &str = match path {
        "/" => "HTTP/1.1 200 OK\r\n\r\n",
        _ => "HTTP/1.1 404 NOT FOUND\r\n\r\n"
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
