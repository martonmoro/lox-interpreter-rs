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

    // declaration    → classDecl | funDecl | varDecl | statement ;
    fn declaration(&mut self) -> Result<Stmt, Error> {
        let statement = if matches!(self, TokenType::Var) {
            self.var_declaration()
        } else if matches!(self, TokenType::Class) {
            self.class_declaration()
        } else if matches!(self, TokenType::Fun) {
            self.function("function")
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

    // classDecl      → "class" IDENTIFIER ( "<" IDENTIFIER )? "{" function* "}" ;
    fn class_declaration(&mut self) -> Result<Stmt, Error> {
        let name = self.consume(TokenType::Identifier, "Expect class name.")?;
        let superclass = if matches!(self, TokenType::Less) {
            self.consume(TokenType::Identifier, "Expect superclass name.")?;
            Some(self.previous().clone())
        } else {
            None
        };
        self.consume(TokenType::LeftBrace, "Expect '{' before class body.")?;

        let mut methods: Vec<Stmt> = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            methods.push(self.function("method")?);
        }

        self.consume(TokenType::RightBrace, "Expect '}' after class body.")?;

        Ok(Stmt::Class {
            name,
            superclass: superclass.map(|name| Expr::Variable { name }),
            methods,
        })
    }

    // Like most dynamically typed languages, fields are not explicitly listed
    // in the class declaration. Instances are loose bags of data and you can
    // freely add fields to them as you see fit using normal imperative code.

    // funDecl        → "fun" function ;
    // function       → IDENTIFIER "(" parameters? ")" block ;
    // parameters     → IDENTIFIER ( "," IDENTIFIER )* ;
    // The parameters rule is like the arguments rule but instead of expressions it has identifiers

    // we’ll reuse the function() method later to parse methods inside classes.
    fn function(&mut self, kind: &str) -> Result<Stmt, Error> {
        // First we consume the identifier token for the function's name
        let name = self.consume(
            TokenType::Identifier,
            format!("Expect {} name.", kind).as_str(),
        )?;

        // Next, we parse the parameter list and the pair of parantheses wrapped around it
        // The result is a list of tokens for each parameter's name
        self.consume(
            TokenType::LeftParen,
            format!("Expect '(' after {} name.", kind).as_str(),
        )?;
        let mut params: Vec<Token> = Vec::new();
        if !self.check(TokenType::RightParen) {
            loop {
                if params.len() >= 255 {
                    // No error returned
                    self.error(self.peek(), "Can't have more than 255 parameters.");
                }

                params.push(self.consume(TokenType::Identifier, "Expect parameter name.")?);

                if !matches!(self, TokenType::Comma) {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen, "Expect ')' after parameters.")?;

        // Finally we parse the body and wrap it all up in a funciton node
        self.consume(
            TokenType::LeftBrace,
            format!("Expected '{{' before {} body", kind).as_str(),
        )?;
        let body = self.block()?;
        Ok(Stmt::Function { name, params, body })
    }

    // statement      → exprStmt | printStmt | ifStmt | block | returnStmt | whileStmt | forStmt ;
    fn statement(&mut self) -> Result<Stmt, Error> {
        if matches!(self, TokenType::For) {
            self.for_statement()
        } else if matches!(self, TokenType::If) {
            self.if_statement()
        } else if matches!(self, TokenType::Print) {
            self.print_statement()
        } else if matches!(self, TokenType::Return) {
            self.return_statement()
        } else if matches!(self, TokenType::While) {
            self.while_statement()
        } else if matches!(self, TokenType::LeftBrace) {
            Ok(Stmt::Block {
                statements: self.block()?,
            })
        } else {
            self.expression_statement()
        }
    }

    // In Lox, the body of a function is a list of statements which don’t produce values, so we need dedicated syntax for emitting a result.
    // returnStmt     → "return" expression? ";" ;
    fn return_statement(&mut self) -> Result<Stmt, Error> {
        let keyword = self.previous().clone();
        let value = if !self.check(TokenType::Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(TokenType::Semicolon, "Expect ';' after return value.")?;
        Ok(Stmt::Return { keyword, value })
    }

    // the else is bound to the nearest if that precedes it
    // ifStmt         → "if" "(" expression ")" statement ( "else" statement )? ;
    fn if_statement(&mut self) -> Result<Stmt, Error> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after if condition.")?;

        let then_branch = Box::new(self.statement()?);

        let else_branch = Box::new(if matches!(self, TokenType::Else) {
            Some(self.statement()?)
        } else {
            None
        });

        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
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

    // whileStmt      → "while" "(" expression ")" statement ;
    fn while_statement(&mut self) -> Result<Stmt, Error> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after condition.")?;
        let body = self.statement()?;

        Ok(Stmt::While {
            condition,
            body: Box::new(body),
        })
    }

    // forStmt        → "for" "(" ( varDecl | exprStmt | ";" ) expression? ";" expression? ")" statement ;
    fn for_statement(&mut self) -> Result<Stmt, Error> {
        self.consume(TokenType::LeftParen, "Expected '(' after 'for'.")?;

        let initializer = if matches!(self, TokenType::Semicolon) {
            None
        } else if matches!(self, TokenType::Var) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let condition = if !self.check(TokenType::Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(TokenType::Semicolon, "Expect ';' after loop condition.")?;

        let increment = if !self.check(TokenType::RightParen) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(TokenType::RightParen, "Expect ')' after for clauses.")?;

        let mut body = self.statement()?;

        if let Some(incr) = increment {
            let incr_stmt = Stmt::Expression { expression: incr };
            body = Stmt::Block {
                statements: vec![body, incr_stmt],
            }
        }

        body = Stmt::While {
            condition: condition.unwrap_or(Expr::Literal {
                value: LiteralValue::Boolean(true),
            }),
            body: Box::new(body),
        };

        if let Some(init) = initializer {
            body = Stmt::Block {
                statements: vec![init, body],
            };
        }

        Ok(body)
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

    // Unlike getters, setters don’t chain. However, the reference to call
    // allows any high-precedence expression before the last dot, including any
    // number of getters,

    // assignment     → ( call "." )? IDENTIFIER "=" assignment| logic_or ;
    fn assignment(&mut self) -> Result<Expr, Error> {
        let expr = self.logic_or()?;

        if matches!(self, TokenType::Equal) {
            // contrary to binary operators we don't loop to build up a sequence of the same operator
            // since assignment is right-associative, we instead recurisvely call assignment() to parse the right hand side
            let value = Box::new(self.assignment()?);

            if let Expr::Variable { name } = expr {
                return Ok(Expr::Assign { name, value });
            } else if let Expr::Get { object, name } = expr {
                return Ok(Expr::Set {
                    object,
                    name,
                    value,
                });
            }

            let equals = self.previous();
            // we are not throwing because the parser is not in a confused state where we need to go into panic mode and synchronize
            self.error(equals, "Invalid assignment target.");
        }

        Ok(expr)
    }

    //logic_or       → logic_and ( "or" logic_and )* ;
    fn logic_or(&mut self) -> Result<Expr, Error> {
        let mut expr = self.logic_and()?;

        while matches!(self, TokenType::Or) {
            let operator = (*self.previous()).clone();
            let right = self.logic_and()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        Ok(expr)
    }

    // logic_and      → equality ( "and" equality )* ;
    fn logic_and(&mut self) -> Result<Expr, Error> {
        let mut expr = self.equality()?;

        while matches!(self, TokenType::Or) {
            let operator = (*self.previous()).clone();
            let right = self.equality()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
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

    // unary          → ( "!" | "-" ) unary | call ;
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

        self.call()
    }

    // call           → primary ( "(" arguments? ")" | "." IDENTIFIER )* ;
    // This rule matches a primary expression followed by zero or more function calls.
    // If there are no parentheses, this parses a bare primary expression.
    // Otherwise, each call is recognized by a pair of parentheses with an optional list of arguments inside.
    fn call(&mut self) -> Result<Expr, Error> {
        let mut expr = self.primary()?;

        loop {
            if matches!(self, TokenType::LeftParen) {
                expr = self.finish_call(expr)?;
            } else if matches!(self, TokenType::Dot) {
                let name = self.consume(TokenType::Identifier, "Expect property after '.'.")?;
                expr = Expr::Get {
                    object: Box::new(expr),
                    name: name,
                }
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, calle: Expr) -> Result<Expr, Error> {
        let mut arguments: Vec<Expr> = Vec::new();
        if !self.check(TokenType::RightParen) {
            loop {
                if arguments.len() >= 255 {
                    // Only reporting error, not throwing.
                    // Throwing is how we kick into panic mode which is what we want if the parser is in a confused state and doesn't know where it is in the grammar anymore.
                    // But here, the parser is still in a prefectly valid state - it just found too many arguments.
                    self.error(self.peek(), "Can't have more than 255 arguments.");
                }

                arguments.push(self.expression()?);

                if !matches!(self, TokenType::Comma) {
                    break;
                }
            }
        }

        let paren = self.consume(TokenType::RightParen, "Expect ')' after arguments.")?;

        Ok(Expr::Call {
            callee: Box::new(calle),
            paren,
            arguments,
        })
    }

    // The argument list grammar is: arguments      → expression ( "," expression )* ;
    // This rule requires at least one argument expression, followed by zero or more other expressions, each preceded by a comma.
    // To handle zero-argument calls, the call rule itself considers the entire arguments production to be optional.

    // primary        → NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")" | IDENTIFIER | "super" "." IDENTIFIER ;
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
            TokenType::This => Expr::This {
                keyword: self.peek().clone(),
            },
            TokenType::Super => {
                let keyword = self.advance().clone();
                self.consume(TokenType::Dot, "Expect '.' after 'super'.")?;
                let method =
                    self.consume(TokenType::Identifier, "Expect superclass method name.")?;
                return Ok(Expr::Super { keyword, method });
            }
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
