use std::fmt;

#[derive(Debug, Clone)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String { literal: String },
    Number { literal: f64 },

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

// we are building the hashmap at compile time
include!(concat!(env!("OUT_DIR"), "/keywords.rs"));

pub struct Token {
    token_type: TokenType,
    lexeme: String,
    line: i32,
    // in the original code it has the literals here but we can encode them in enums so we don't have to store the separately
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: &str, line: i32) -> Self {
        Self {
            token_type,
            lexeme: lexeme.to_string(),
            line,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!("Implement when I have literals figured out")
    }
}
