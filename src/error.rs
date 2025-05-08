use std::convert;
use std::fmt;
use std::io;

use crate::object::Object;
use crate::token::{Token, TokenType};

pub fn error(line: i32, message: &str) {
    report(line, "", message);
}

fn report(line: i32, where_: &str, message: &str) {
    eprintln!("[line {line}] Error{where_}: {message}");
    //hadError = true;
}

pub fn parser_error(token: &Token, msg: &str) {
    if token.token_type == TokenType::Eof {
        report(token.line, " at end", msg);
    } else {
        report(token.line, &format!(" at '{}'", token.lexeme), msg);
    }
}

#[derive(Debug)]
pub enum Error {
    Parse,
    Return { value: Object },
    Runtime { token: Token, message: String },
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(underlying) => write!(f, "IoError {}", underlying),
            Error::Parse => write!(f, "ParseError"),
            Error::Return { value } => write!(f, "Return {:?}", value),
            Error::Runtime { message, .. } => write!(f, "RuntimeError {}", message),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        "Lox Error"
    }
}

impl convert::From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}
