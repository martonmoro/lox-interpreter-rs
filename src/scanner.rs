// A lexeme is the raw sequence of characters in the source code that represents a meaningful unit
// A token is a categorized representation of a lexeme, pairing it with its type

use crate::error::error;
use crate::token::{Token, TokenType, KEYWORDS};

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: i32,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Self {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> &Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()
        }

        self.tokens.push(Token::new(TokenType::Eof, "", self.line));
        &self.tokens
    }

    fn scan_token(&mut self) {
        let c: char = self.advance();
        match c {
            // single char
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::Semicolon),
            '*' => self.add_token(TokenType::Star),

            // can be double char
            '!' => {
                if self.r#match('=') {
                    self.add_token(TokenType::BangEqual);
                } else {
                    self.add_token(TokenType::Bang);
                }
            }
            '=' => {
                if self.r#match('=') {
                    self.add_token(TokenType::EqualEqual);
                } else {
                    self.add_token(TokenType::Equal);
                }
            }
            '<' => {
                if self.r#match('=') {
                    self.add_token(TokenType::LessEqual);
                } else {
                    self.add_token(TokenType::Less);
                }
            }
            '>' => {
                if self.r#match('=') {
                    self.add_token(TokenType::GreaterEqual);
                } else {
                    self.add_token(TokenType::Greater);
                }
            }

            // can be comment
            '/' => {
                if self.r#match('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash);
                }
            }

            // ignore whitespace
            ' ' | '\t' | '\r' => (),

            // handle new line
            '\n' => {
                self.line += 1;
            }

            '"' => self.string(),

            c => {
                if c.is_digit(10) {
                    self.number()
                } else if c.is_alphabetic() || c == '_' {
                    self.identifier()
                } else {
                    error(self.line, "Unexpected character.")
                }
            }
        }
    }

    // consume characters until we reach the closing "
    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            error(self.line, "Unterminated string");
        }

        // the closing "
        self.advance();

        // trim
        let literal = self
            .source
            .get((self.start + 1)..(self.current - 1))
            .expect("Unexpected string end.")
            .to_string();

        self.add_token(TokenType::String { literal });
    }

    fn number(&mut self) {
        while self.peek().is_digit(10) {
            self.advance();
        }

        // consume the .
        if self.peek() == '.' && self.peek_next().is_digit(10) {
            self.advance();

            while self.peek().is_digit(10) {
                self.advance();
            }
        }

        let literal: f64 = self
            .source
            .get(self.start..self.current)
            .expect("Unexpected number end")
            .parse() // we could do .parse::<64> using the turbofish
            .expect("Scanned number could not be parsed");

        self.add_token(TokenType::Number { literal });
    }

    fn identifier(&mut self) {
        while self.peek().is_alphanumeric() || self.peek() == '_' {
            self.advance();
        }

        let text = self
            .source
            .get(self.start..self.current)
            .expect("Unexpected identifier end.");
        let tpe = KEYWORDS.get(text).cloned().unwrap_or(TokenType::Identifier);

        self.add_token(tpe);
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        return self
            .source
            .chars()
            .nth(self.current - 1)
            .expect("there is a next char");
    }

    // it's like advance but doesn't consume the next character
    fn peek(&self) -> char {
        self.source.chars().nth(self.current).unwrap_or('\0')
    }

    fn peek_next(&self) -> char {
        self.source.chars().nth(self.current + 1).unwrap_or('\0')
    }

    fn add_token(&mut self, token_type: TokenType) {
        let text = self
            .source
            .get(self.start..self.current)
            .expect("Source token is empty");
        self.tokens.push(Token::new(token_type, text, self.line));
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    // we only consume the current character if that is what we are looking for
    fn r#match(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }

        if self
            .source
            .chars()
            .nth(self.current)
            .expect("Unexpected EOF")
            != expected
        {
            return false;
        }

        self.current += 1;
        true
    }
}
