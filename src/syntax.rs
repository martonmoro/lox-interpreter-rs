use std::fmt;

use crate::error::Error;
use crate::token::Token;

// we don't really need to generate these like they are generated using a script in the book
#[derive(Debug)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Literal {
        value: LiteralValue,
    },
    Variable {
        name: Token,
    },
    Assign {
        name: Token,
        value: Box<Expr>,
    },
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub enum LiteralValue {
    Boolean(bool),
    Number(f64),
    Null,
    String(String),
}

impl fmt::Display for LiteralValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LiteralValue::Boolean(b) => write!(f, "{}", b),
            LiteralValue::Null => write!(f, "null"),
            LiteralValue::Number(n) => write!(f, "{}", n),
            LiteralValue::String(s) => write!(f, "{}", s),
        }
    }
}

impl Expr {
    // we could have used an opaque type pub fn accept<R>(&self, visitor: &impl Visitor<R>) -> R
    // or dynamic dispatch pub fn accept<R>(&self, visitor: &dyn Visitor<R>) -> R
    // instead of the trait bound
    pub fn accept<R, T: expr::Visitor<R>>(&self, visitor: &T) -> Result<R, Error> {
        match self {
            Expr::Binary {
                left,
                operator,
                right,
            } => visitor.visit_binary_expr(left, operator, right),
            Expr::Grouping { expression } => visitor.visit_grouping_expr(expression),
            Expr::Literal { value } => visitor.visit_literal_expr(value),
            Expr::Unary { operator, right } => visitor.visit_unary_expr(operator, right),
            Expr::Variable { name } => visitor.visit_variable_expr(name),
            Expr::Assign { name, value } => visitor.visit_assign_expr(name, value),
        }
    }
}

pub mod expr {
    use crate::error::Error;
    use crate::token::Token;

    use super::{Expr, LiteralValue};

    pub trait Visitor<R> {
        fn visit_binary_expr(
            &self,
            left: &Expr,
            operator: &Token,
            right: &Expr,
        ) -> Result<R, Error>;
        fn visit_grouping_expr(&self, expression: &Expr) -> Result<R, Error>;
        fn visit_literal_expr(&self, value: &LiteralValue) -> Result<R, Error>;
        fn visit_unary_expr(&self, operator: &Token, right: &Expr) -> Result<R, Error>;
        fn visit_variable_expr(&self, name: &Token) -> Result<R, Error>;
        fn visit_assign_expr(&self, name: &Token, value: &Expr) -> Result<R, Error>;
    }
}
#[derive(Debug)]
pub enum Stmt {
    Expression {
        expression: Expr,
    },
    Print {
        expression: Expr,
    },
    Var {
        name: Token,
        initializer: Option<Expr>,
    },
    Null, // placeholder until statement handling is figured out after synchronize()
}

impl Stmt {
    pub fn accept<R, T: stmt::Visitor<R>>(&self, visitor: &T) -> Result<R, Error> {
        match self {
            Stmt::Expression { expression } => visitor.visit_expression_stmt(expression),
            Stmt::Print { expression } => visitor.visit_print_stmt(expression),
            Stmt::Var { name, initializer } => visitor.visit_var_stmt(name, initializer),
            Stmt::Null => unimplemented!(),
        }
    }
}

pub mod stmt {
    use crate::error::Error;
    use crate::token::Token;

    use super::{Expr, Stmt};

    pub trait Visitor<R> {
        fn visit_expression_stmt(&self, stmt: &Expr) -> Result<R, Error>;
        fn visit_print_stmt(&self, stmt: &Expr) -> Result<R, Error>;
        fn visit_var_stmt(&self, name: &Token, initializer: &Option<Expr>) -> Result<R, Error>;
    }
}

pub struct AstPrinter;

impl AstPrinter {
    pub fn print(&self, expr: Expr) -> Result<String, Error> {
        expr.accept(self)
    }

    fn paranthesize(&self, name: String, exprs: Vec<&Expr>) -> Result<String, Error> {
        let mut builder = String::new();

        builder.push_str("(");
        builder.push_str(&name);

        for expr in exprs {
            builder.push_str(" ");
            builder.push_str(&expr.accept(self)?);
        }

        builder.push_str(")");

        Ok(builder)
    }
}

impl expr::Visitor<String> for AstPrinter {
    fn visit_binary_expr(
        &self,
        left: &Expr,
        operator: &Token,
        right: &Expr,
    ) -> Result<String, Error> {
        self.paranthesize(operator.lexeme.clone(), vec![left, right])
    }

    fn visit_grouping_expr(&self, expression: &Expr) -> Result<String, Error> {
        self.paranthesize("group".to_string(), vec![expression])
    }

    fn visit_literal_expr(&self, value: &LiteralValue) -> Result<String, Error> {
        Ok(value.to_string())
    }

    fn visit_unary_expr(&self, operator: &Token, right: &Expr) -> Result<String, Error> {
        self.paranthesize(operator.lexeme.clone(), vec![right])
    }

    fn visit_variable_expr(&self, name: &Token) -> Result<String, Error> {
        Ok(name.lexeme.clone())
    }
}

// test from the book
#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::{Token, TokenType};

    #[test]
    fn test_printer() {
        let expression = Expr::Binary {
            left: Box::new(Expr::Unary {
                operator: Token::new(TokenType::Minus, "-", 1),
                right: Box::new(Expr::Literal {
                    value: LiteralValue::Number(123f64),
                }),
            }),
            operator: Token::new(TokenType::Star, "*", 1),
            right: Box::new(Expr::Grouping {
                expression: Box::new(Expr::Literal {
                    value: LiteralValue::Number(45.67),
                }),
            }),
        };
        let printer = AstPrinter;

        assert_eq!(
            printer.print(expression).unwrap(),
            "(* (- 123) (group 45.67))"
        );
    }
}
