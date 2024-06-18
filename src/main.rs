use std::{
    io::{Read, Write},
    net::TcpListener,
    thread,
};

mod command;
mod parser;

fn handle_connection<S: Read + Write>(mut stream: S) {
    let mut buf = [0; 512];

    loop {
        let n = stream.read(&mut buf).expect("Failed to read from client");
        if n == 0 {
            break;
        }

        let request = parser::parse_iter(&mut buf.iter());
        let cmd = command::build_cmd(request);
        let msg = cmd.handle();

        stream.write_all(&msg).expect("Fail to write to client");
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
    use command::Command;
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
        let ping = command::Ping::new("".as_bytes());
        let mut mock_stream = MockStream::new();

        mock_stream.add_to_read(&ping.to_vec());
        handle_connection(&mut mock_stream);
        assert_eq!(mock_stream.writter, "+PONG\r\n".as_bytes());
    }

    #[test]
    fn test_handle_echo() {
        let echo = command::Echo::new("hey".as_bytes());
        let mut mock_stream = MockStream::new();

        mock_stream.add_to_read(&echo.to_vec());
        handle_connection(&mut mock_stream);
        assert_eq!(mock_stream.writter, "$3\r\nhey\r\n".as_bytes());
    }
}
