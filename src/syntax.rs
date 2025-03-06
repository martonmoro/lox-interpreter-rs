use std::fmt;

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

pub trait Visitor<R> {
    fn visit_binary_expr(&self, left: &Expr, operator: &Token, right: &Expr) -> R;
    fn visit_grouping_expr(&self, expression: &Expr) -> R;
    fn visit_literal_expr(&self, value: String) -> R;
    fn visit_unary_expr(&self, operator: &Token, right: &Expr) -> R;
}

impl Expr {
    // we could have used an opaque type pub fn accept<R>(&self, visitor: &impl Visitor<R>) -> R
    // or dynamic dispatch pub fn accept<R>(&self, visitor: &dyn Visitor<R>) -> R
    // instead of the trait bound
    pub fn accept<R, T: Visitor<R>>(&self, visitor: &T) -> R {
        match self {
            Expr::Binary {
                left,
                operator,
                right,
            } => visitor.visit_binary_expr(left, operator, right),
            Expr::Grouping { expression } => visitor.visit_grouping_expr(expression),
            Expr::Literal { value } => visitor.visit_literal_expr(value.to_string()),
            Expr::Unary { operator, right } => visitor.visit_unary_expr(operator, right),
        }
    }
}

pub struct AstPrinter;

impl AstPrinter {
    pub fn print(&self, expr: Expr) -> String {
        expr.accept(self)
    }

    fn paranthesize(&self, name: String, exprs: Vec<&Expr>) -> String {
        let mut builder = String::new();

        builder.push_str("(");
        builder.push_str(&name);

        for expr in exprs {
            builder.push_str(" ");
            builder.push_str(&expr.accept(self));
        }

        builder.push_str(")");

        builder
    }
}

impl Visitor<String> for AstPrinter {
    fn visit_binary_expr(&self, left: &Expr, operator: &Token, right: &Expr) -> String {
        self.paranthesize(operator.lexeme.clone(), vec![left, right])
    }

    fn visit_grouping_expr(&self, expression: &Expr) -> String {
        self.paranthesize("group".to_string(), vec![expression])
    }

    fn visit_literal_expr(&self, value: String) -> String {
        value
    }

    fn visit_unary_expr(&self, operator: &Token, right: &Expr) -> String {
        self.paranthesize(operator.lexeme.clone(), vec![right])
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

        assert_eq!(printer.print(expression), "(* (- 123) (group 45.67))");
    }
}
