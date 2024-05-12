use std::{
    io::{Read, Write},
    net::TcpListener,
    thread,
};

fn handle_connection<S: Read + Write>(mut stream: S) {
    let mut buf = [0; 512];

    loop {
        let n = stream.read(&mut buf).expect("Failed to read from client");
        if n == 0 {
            break;
        }

        let msg = b"+PONG\r\n".to_vec();

        stream.write_all(&msg).expect("Fail to write ti client");
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use std::io::{Cursor, Read, Write};

    struct MockStream {
        reader: Cursor<Vec<u8>>,
        writter: Vec<u8>,
    }

    impl MockStream {
        fn new() -> MockStream {
            MockStream {
                reader: Cursor::new(vec![]),
                writter: vec![],
            }
        }

        fn add_to_read(&mut self, buf: &[u8]) {
            self.reader.get_mut().extend(buf)
        }
    }

    impl Read for MockStream {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            self.reader.read(buf)
        }
    }

    impl Write for MockStream {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.writter.write(buf)
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_handle_ping() {
        let vec = "PING".to_string().as_bytes().to_owned();
        let mut mock_stream = MockStream::new();

        mock_stream.add_to_read(&vec);
        handle_connection(&mut mock_stream);
        assert_eq!(mock_stream.writter, "+PONG\r\n".as_bytes());
    }
}
