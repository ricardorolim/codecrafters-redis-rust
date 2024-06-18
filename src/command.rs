use core::panic;

use crate::parser::Element;

pub trait Command {
    fn handle(&self) -> Vec<u8>;
    fn to_vec(&self) -> Vec<u8>;
}

pub fn build_cmd(array: Element) -> Box<dyn Command> {
    match array {
        Element::Array(elems) => match &elems[0] {
            Element::BulkString(vec) => {
                let arg = if elems.len() == 1 {
                    "".as_bytes().to_vec()
                } else {
                    match &elems[1] {
                        Element::BulkString(arg) => arg.clone(),
                        _ => panic!(),
                    }
                };

                let str = &String::from_utf8(vec.to_vec()).unwrap()[..];
                match str {
                    PING_CMD => Box::new(Ping::new(&arg)),
                    ECHO_CMD => Box::new(Echo::new(&arg)),
                    _ => panic!("unknown command: {}", str),
                }
            }
            _ => panic!(),
        },
        _ => panic!(),
    }
}

const PING_CMD: &str = "PING";
const ECHO_CMD: &str = "ECHO";

pub struct Echo {
    message: Vec<u8>,
}

impl Echo {
    pub fn new(message: &[u8]) -> Self {
        Echo {
            message: message.to_vec(),
        }
    }
}

impl Command for Echo {
    fn handle(&self) -> Vec<u8> {
        Element::BulkString(self.message.clone()).to_vec()
    }

    fn to_vec(&self) -> Vec<u8> {
        let cmd = Element::BulkString(ECHO_CMD.as_bytes().to_vec());
        let arg = Element::BulkString(self.message.clone());
        let array = Element::Array(vec![cmd, arg]);
        array.to_vec()
    }
}

pub struct Ping {
    message: Vec<u8>,
}

impl Ping {
    pub fn new(message: &[u8]) -> Self {
        Ping {
            message: message.to_vec(),
        }
    }
}

impl Command for Ping {
    fn handle(&self) -> Vec<u8> {
        let resp = if self.message.len() == 0 {
            Element::String("PONG".to_string())
        } else {
            Element::BulkString(self.message.clone())
        };

        resp.to_vec()
    }

    fn to_vec(&self) -> Vec<u8> {
        let cmd = Element::BulkString(PING_CMD.as_bytes().to_vec());

        let arg = Element::BulkString(self.message.clone());
        let elems = if self.message.len() > 0 {
            vec![cmd, arg]
        } else {
            vec![cmd]
        };

        Element::Array(elems).to_vec()
    }
}
