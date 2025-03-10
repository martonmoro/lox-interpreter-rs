use crate::error::{parser_error, Error};

use crate::syntax::{Expr, LiteralValue, Stmt};
use crate::token::{Token, TokenType};

pub struct Parser<'t> {
    tokens: &'t Vec<Token>,
    current: usize,
}

macro_rules! matches {
    ( $sel:ident, $( $x:expr ),* ) => {
        {
            if $( $sel.check($x) )||* {
                $sel.advance();
                true
            } else {
                false
            }
        }
    };
}

impl<'t> Parser<'t> {
    pub fn new(tokens: &'t Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }
    // program        → declaration* EOF ;
    pub fn parse(&mut self) -> Result<Vec<Stmt>, Error> {
        let mut statements: Vec<Stmt> = Vec::new();
        while !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        Ok(statements)
    }

    // declaration    → varDecl | statement ;
    fn declaration(&mut self) -> Result<Stmt, Error> {
        let statement = if matches!(self, TokenType::Var) {
            self.var_declaration()
        } else {
            self.statement()
        };

        // catch the "exception thrown" when the parser begins error recovery
        match statement {
            Err(Error::Parse) => {
                self.synchronize();
                Ok(Stmt::Null)
            }
            other => other,
        }
    }

    // statement      → exprStmt | printStmt | block ;
    fn statement(&mut self) -> Result<Stmt, Error> {
        if matches!(self, TokenType::Print) {
            self.print_statement()
        } else if matches!(self, TokenType::LeftBrace) {
            Ok(Stmt::Block {
                statements: self.block()?,
            })
        } else {
            self.expression_statement()
        }
    }

    // block          → "{" declaration* "}" ;
    fn block(&mut self) -> Result<Vec<Stmt>, Error> {
        let mut statements: Vec<Stmt> = Vec::new();

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.")?;
        Ok(statements)
    }

    // varDecl        → "var" IDENTIFIER ( "=" expression )? ";" ;
    fn var_declaration(&mut self) -> Result<Stmt, Error> {
        let name = self.consume(TokenType::Identifier, "Expected variable name.")?;
        let initializer = if matches!(self, TokenType::Equal) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(
            TokenType::Semicolon,
            "Expected ; after variable declaration.",
        )?;

        Ok(Stmt::Var { name, initializer })
    }

    // expression     → assignment ;
    fn expression(&mut self) -> Result<Expr, Error> {
        self.assignment()
    }

    // The trick is that the parser first processes the left side as it it were an expression (r-value),
    // then converts it to an assignment target (l-value) if an = sign follows
    // This conversion works because it turns out that every valid assignment target happens to also be valid syntax as a normal expression.
    // assignment     → IDENTIFIER "=" assignment | equality ;
    fn assignment(&mut self) -> Result<Expr, Error> {
        let expr = self.equality()?;

        if matches!(self, TokenType::Equal) {
            // contrary to binary operators we don't loop to build up a sequence of the same operator
            // since assignment is right-associative, we instead recurisvely call assignment() to parse the right hand side
            let value = Box::new(self.assignment()?);

            if let Expr::Variable { name } = expr {
                return Ok(Expr::Assign { name, value });
            }

            let equals = self.previous();
            // we are not throwing because the parser is not in a confused state where we need to go into panic mode and synchronize
            self.error(equals, "Invalid assignment target.");
        }

        Ok(expr)
    }

    // equality       → comparison ( ( "!=" | "==" ) comparison )* ;
    /*
       If the parser never encounters an equality operator, then it never enters the loop.
       In that case, the equality() method effectively calls and returns comparison().
       In that way, this method matches an equality operator or anything of higher precedence.
    */
    fn equality(&mut self) -> Result<Expr, Error> {
        // the first comparison nonterminal in the body
        let mut expr: Expr = self.comparison()?;

        while matches!(self, TokenType::BangEqual, TokenType::EqualEqual) {
            // we know we are parsing an equality expression
            // we grab the matched operator token
            let operator = (*self.previous()).clone();
            // parse the right hand operand
            let right: Expr = self.comparison()?;
            // For each iteration, we create a new binary expression using the previous one as the left operand.
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    // comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
    fn comparison(&mut self) -> Result<Expr, Error> {
        let mut expr: Expr = self.term()?;

        while matches!(
            self,
            TokenType::GreaterEqual,
            TokenType::Greater,
            TokenType::LessEqual,
            TokenType::Less
        ) {
            let operator = (*self.previous()).clone();
            let right: Expr = self.term()?;

            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        Ok(expr)
    }

    // term           → factor ( ( "-" | "+" ) factor )* ;
    fn term(&mut self) -> Result<Expr, Error> {
        let mut expr: Expr = self.factor()?;

        while matches!(self, TokenType::Minus, TokenType::Plus) {
            let operator = (*self.previous()).clone();
            let right: Expr = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    // factor         → unary ( ( "/" | "*" ) unary )* ;
    fn factor(&mut self) -> Result<Expr, Error> {
        let mut expr: Expr = self.unary()?;

        while matches!(self, TokenType::Slash, TokenType::Star) {
            print!("In the while in factor");
            let operator = (*self.previous()).clone();
            let right: Expr = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        Ok(expr)
    }

    // unary          → ( "!" | "-" ) unary | primary ;
    fn unary(&mut self) -> Result<Expr, Error> {
        if matches!(self, TokenType::Bang, TokenType::Minus) {
            let operator = (*self.previous()).clone();
            let right = self.unary()?;
            let expr = Expr::Unary {
                operator,
                right: Box::new(right),
            };
            return Ok(expr);
        }

        self.primary()
    }

    // primary        → NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")" | IDENTIFIER ;
    fn primary(&mut self) -> Result<Expr, Error> {
        let expr = match &self.peek().token_type {
            TokenType::False => Expr::Literal {
                value: LiteralValue::Boolean(false),
            },
            TokenType::True => Expr::Literal {
                value: LiteralValue::Boolean(true),
            },
            TokenType::Nil => Expr::Literal {
                value: LiteralValue::Null,
            },
            TokenType::Number { literal } => Expr::Literal {
                value: LiteralValue::Number(literal.clone()),
            },
            TokenType::String { literal } => Expr::Literal {
                value: LiteralValue::String(literal.clone()),
            },
            TokenType::LeftParen => {
                let expr = self.expression()?;
                self.consume(TokenType::RightParen, "Expect ')' after expression.")?;
                Expr::Grouping {
                    expression: Box::new(expr),
                }
            }
            TokenType::Identifier => Expr::Variable {
                name: self.peek().clone(),
            },
            _ => return Err(self.error(self.peek(), "Expect expression")),
        };

        self.advance();

        Ok(expr)
    }

    // printStmt      → "print" expression ";" ;
    fn print_statement(&mut self) -> Result<Stmt, Error> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expected ; after value.")?;
        Ok(Stmt::Print { expression: value })
    }

    // exprStmt       → expression ";" ;
    fn expression_statement(&mut self) -> Result<Stmt, Error> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expected ; after value.")?;
        Ok(Stmt::Expression { expression: value })
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type == TokenType::Semicolon {
                return;
            }

            match self.peek().token_type {
                TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => self.advance(),
            };
        }
    }

    // leaving this here to match the book but it is better with the macro
    // fn matches(&mut self, types: Vec<TokenType>) -> bool {
    //     for tpe in types {
    //         if self.check(tpe) {
    //             self.advance();
    //             return true;
    //         }
    //     }
    //     false
    // }

    // returns true if the current token is of the given type. Unlike match(), it never consumes the token, it only looks at it.
    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().token_type == token_type
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        return self.peek().token_type == TokenType::Eof;
    }

    fn peek(&self) -> &Token {
        self.tokens
            .get(self.current)
            .expect("Peek into end of token stream.")
    }

    fn previous(&self) -> &Token {
        self.tokens
            .get(self.current - 1)
            .expect("Previous was empty.")
    }

    fn consume(&mut self, token_type: TokenType, msg: &str) -> Result<Token, Error> {
        if self.check(token_type) {
            Ok(self.advance().clone())
        } else {
            Err(self.error(self.peek(), msg))
        }
    }

    fn error(&self, token: &Token, msg: &str) -> Error {
        parser_error(token, msg);
        Error::Parse
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::Scanner;
    use crate::syntax::AstPrinter;

    #[test]
    fn test_parser() {
        let mut scanner = Scanner::new("-123 * 45.67".to_string());
        let tokens = scanner.scan_tokens();

        let mut parser = Parser::new(tokens);
        let statements = parser.parse().expect("Could not parse sample code.");
        let printer = AstPrinter;

        // assert_eq!(printer.print(statements).unwrap(), "(* (- 123) 45.67)");
    }
}
