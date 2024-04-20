use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

fn handle_connection(mut stream: TcpStream) {
    let mut buf = [0; 512];
    let n = stream.read(&mut buf).expect("Failed to read from client");
    println!("received {} bytes", n);
    println!("data {:?}", &buf[..n]);

    stream
        .write_all(b"+PONG\r\n")
        .expect("Fail to write to client");
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

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
