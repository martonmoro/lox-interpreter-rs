use std::fmt;

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
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Parse => write!(f, "ParseError"),
        }
    }
}
