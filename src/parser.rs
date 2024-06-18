use core::panic;
use std::io::Write;
use std::slice::Iter;
use std::u8;

#[derive(Debug)]
struct ParseError;

type Result<T> = std::result::Result<T, ParseError>;

#[derive(Debug, PartialEq)]
pub enum Element {
    String(String),
    Error(String),
    Integer(i32),
    BulkString(Vec<u8>),
    Array(Vec<Element>),
    Null,
    Boolean(bool),
}

impl Element {
    pub fn to_vec(&self) -> Vec<u8> {
        match self {
            Self::String(str) => {
                let mut w = Vec::new();
                write!(&mut w, "+{}\r\n", str).unwrap();
                w
            }
            Self::BulkString(str) => {
                let mut w = Vec::new();
                write!(&mut w, "${}\r\n", str.len()).unwrap();
                w.extend_from_slice(str);
                write!(&mut w, "\r\n").unwrap();
                w
            }
            Self::Array(elems) => {
                let mut w = Vec::new();
                let vec: Vec<u8> = elems.iter().flat_map(|e| e.to_vec()).collect();
                write!(&mut w, "*{}\r\n", elems.len()).unwrap();
                w.extend_from_slice(&vec);
                w
            }
            _ => vec![],
        }
    }
}

fn parse_bulk_string(bytes: &mut Iter<u8>) -> Element {
    let length = extract_length(bytes);
    let data = bytes.take(length).copied().collect();
    discard_terminator(bytes);
    Element::BulkString(data)
}

fn extract_length(bytes: &mut Iter<u8>) -> usize {
    let length = until_newline(bytes).expect("unterminated bulk string");
    let length = String::from_utf8(length).expect("invalid bulk string");
    length.parse().expect("invalid bulk string length")
}

fn parse_simple_string(bytes: &mut Iter<u8>) -> Element {
    let data = until_newline(bytes).expect("unterminated string");
    let string = String::from_utf8(data).expect("invalid simple string");
    Element::String(string)
}

fn parse_simple_error(bytes: &mut Iter<u8>) -> Element {
    let data = until_newline(bytes).expect("unterminated string");
    let string = String::from_utf8(data).expect("invalid simple string");
    Element::Error(string)
}

fn parse_integer(bytes: &mut Iter<u8>) -> Element {
    let data = until_newline(bytes).expect("unterminated integer");
    let integer = String::from_utf8(data).expect("invalid integer");
    Element::Integer(integer.parse().expect("invalid integer"))
}

fn parse_null(bytes: &mut Iter<u8>) -> Element {
    discard_terminator(bytes);
    Element::Null
}

fn parse_boolean(bytes: &mut Iter<u8>) -> Element {
    let data = until_newline(bytes).expect("unterminated integer");
    assert_eq!(data.len(), 1, "boolean size should be 1 character");
    let bool = match data[0] {
        b't' => true,
        b'f' => false,
        _ => panic!("invalid boolean value: {}", data[0]),
    };

    Element::Boolean(bool)
}

fn until_newline(input: &mut Iter<u8>) -> Result<Vec<u8>> {
    let mut prev = b' ';
    let mut line = Vec::new();
    let mut terminated = false;

    for c in input {
        if prev == b'\r' && *c == b'\n' {
            terminated = true;
            break;
        }
        prev = *c;
        line.push(*c);
    }

    if !terminated {
        return Err(ParseError);
    }

    // discard \r
    line.pop();
    Ok(line)
}

pub fn parse_iter(input: &mut Iter<u8>) -> Element {
    let c = input.next().expect("No chars left to parse");
    match c {
        b'+' => parse_simple_string(input),
        b'-' => parse_simple_error(input),
        b':' => parse_integer(input),
        b'$' => parse_bulk_string(input),
        b'*' => parse_array(input),
        b'_' => parse_null(input),
        b'#' => parse_boolean(input),
        _ => panic!("unexpected char: {}", c),
    }
}

fn parse_array(input: &mut Iter<u8>) -> Element {
    let mut elems = Vec::new();
    let length = extract_length(input);

    for _ in 0..length {
        elems.push(parse_iter(input));
    }

    Element::Array(elems)
}

// Discard \r\n terminator from input string
fn discard_terminator(input: &mut Iter<u8>) {
    input.next();
    input.next();
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_simple_string() {
        let string = "+test123\r\n";
        let result = parse_iter(&mut string.as_bytes().iter());
        assert_eq!(result, Element::String("test123".to_string()));
    }

    #[test]
    fn test_parse_simple_errors() {
        let string = "-test123\r\n";
        let result = parse_iter(&mut string.as_bytes().iter());
        assert_eq!(result, Element::Error("test123".to_string()));
    }

    #[test]
    fn test_parse_integer() {
        let string = ":123\r\n";
        let result = parse_iter(&mut string.as_bytes().iter());
        assert_eq!(result, Element::Integer(123));

        let string = ":+123\r\n";
        let result = parse_iter(&mut string.as_bytes().iter());
        assert_eq!(result, Element::Integer(123));

        let string = ":-123\r\n";
        let result = parse_iter(&mut string.as_bytes().iter());
        assert_eq!(result, Element::Integer(-123));
    }

    #[test]
    fn test_bulk_string() {
        let string = "$5\r\nhello\r\n";
        let result = parse_iter(&mut string.as_bytes().iter());
        assert_eq!(result, Element::BulkString("hello".as_bytes().to_vec()));

        let string = "$0\r\n\r\n";
        let result = parse_iter(&mut string.as_bytes().iter());
        assert_eq!(result, Element::BulkString("".as_bytes().to_vec()));
    }

    #[test]
    fn test_array() {
        let string = "*0\r\n";
        let result = parse_iter(&mut string.as_bytes().iter());
        assert_eq!(result, Element::Array(vec![]));

        let string = "*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n";
        let result = parse_iter(&mut string.as_bytes().iter());
        let hello = Element::BulkString("hello".as_bytes().to_vec());
        let world = Element::BulkString("world".as_bytes().to_vec());
        assert_eq!(result, Element::Array(vec![hello, world]));

        let string = "*3\r\n:1\r\n:2\r\n:3\r\n";
        let result = parse_iter(&mut string.as_bytes().iter());
        let one = Element::Integer(1);
        let two = Element::Integer(2);
        let three = Element::Integer(3);
        assert_eq!(result, Element::Array(vec![one, two, three]));

        let string = "*5\r\n:1\r\n:2\r\n:3\r\n:4\r\n$5\r\nhello\r\n";
        let result = parse_iter(&mut string.as_bytes().iter());
        let one = Element::Integer(1);
        let two = Element::Integer(2);
        let three = Element::Integer(3);
        let four = Element::Integer(4);
        let hello = Element::BulkString("hello".as_bytes().to_vec());
        assert_eq!(result, Element::Array(vec![one, two, three, four, hello]));

        let string = "*2\r\n*3\r\n:1\r\n:2\r\n:3\r\n*2\r\n+Hello\r\n-World\r\n";
        let result = parse_iter(&mut string.as_bytes().iter());
        let one = Element::Integer(1);
        let two = Element::Integer(2);
        let three = Element::Integer(3);
        let hello = Element::String("Hello".to_string());
        let world = Element::Error("World".to_string());
        assert_eq!(
            result,
            Element::Array(vec![
                Element::Array(vec![one, two, three]),
                Element::Array(vec![hello, world])
            ])
        );
    }

    #[test]
    fn test_null() {
        let string = "_\r\n";
        let result = parse_iter(&mut string.as_bytes().iter());
        assert_eq!(result, Element::Null);
    }

    #[test]
    fn test_boolean() {
        let string = "#t\r\n";
        let result = parse_iter(&mut string.as_bytes().iter());
        assert_eq!(result, Element::Boolean(true));

        let string = "#f\r\n";
        let result = parse_iter(&mut string.as_bytes().iter());
        assert_eq!(result, Element::Boolean(false));
    }
}
