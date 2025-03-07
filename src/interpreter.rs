use crate::error::Error;
use crate::syntax::{Expr, LiteralValue, Visitor};
use crate::token::{Token, TokenType};

// The book is using java.lang.Object
enum Object {
    Boolean(bool),
    Null,
    Number(f64),
    String(String),
}

impl Object {
    fn equals(&self, other: &Object) -> bool {
        match (self, other) {
            (Object::Null, Object::Null) => true,
            (_, Object::Null) => false,
            (Object::Null, _) => false,
            (Object::Boolean(left), Object::Boolean(right)) => left == right,
            (Object::Number(left), Object::Number(right)) => left == right,
            (Object::String(left), Object::String(right)) => left.eq(right),
            _ => false,
        }
    }
}

pub struct Interpreter {}

impl Interpreter {
    pub fn interpret(&self, expression: &Expr) -> Result<String, Error> {
        self.evaluate(expression).map(|value| self.stringify(value))
    }
    // simply call interpreters visitor implementation
    fn evaluate(&self, expr: &Expr) -> Result<Object, Error> {
        expr.accept(self)
    }

    fn stringify(&self, object: Object) -> String {
        match object {
            Object::Null => "nil".to_string(),
            Object::Number(n) => n.to_string(),
            Object::Boolean(b) => b.to_string(),
            Object::String(s) => s,
        }
    }

    // used like checkNumberOperands in the book
    fn number_operand_error<R>(&self, operator: &Token) -> Result<R, Error> {
        Err(Error::Runtime {
            token: operator.clone(),
            message: "Operand must be a number".to_string(),
        })
    }

    fn is_truthy(&self, right: &Object) -> bool {
        match right {
            Object::Null => false,
            Object::Boolean(b) => b.clone(),
            _ => true,
        }
    }

    fn is_equal(&self, left: &Object, right: &Object) -> bool {
        left.equals(right)
    }
}

impl Visitor<Object> for Interpreter {
    fn visit_literal_expr(&self, value: &LiteralValue) -> Result<Object, Error> {
        // they implement copy
        match value {
            LiteralValue::Boolean(b) => Ok(Object::Boolean(b.clone())),
            LiteralValue::Null => Ok(Object::Null),
            LiteralValue::Number(n) => Ok(Object::Number(n.clone())),
            LiteralValue::String(s) => Ok(Object::String(s.clone())),
        }
    }

    fn visit_grouping_expr(&self, expression: &Expr) -> Result<Object, Error> {
        self.evaluate(expression)
    }

    fn visit_unary_expr(&self, operator: &Token, right: &Expr) -> Result<Object, Error> {
        let right = self.evaluate(right)?;

        match operator.token_type {
            TokenType::Minus => match right {
                Object::Number(n) => Ok(Object::Number(-n)),
                _ => self.number_operand_error(operator),
            },
            TokenType::Bang => Ok(Object::Boolean(!self.is_truthy(&right))),
            _ => unreachable!(),
        }
    }

    fn visit_binary_expr(
        &self,
        left: &Expr,
        operator: &Token,
        right: &Expr,
    ) -> Result<Object, Error> {
        let l = self.evaluate(left)?;
        let r = self.evaluate(right)?;

        match operator.token_type {
            TokenType::Minus => match (l, r) {
                (Object::Number(left_num), Object::Number(right_num)) => {
                    Ok(Object::Number(left_num - right_num))
                }
                _ => self.number_operand_error(operator),
            },
            TokenType::Slash => match (l, r) {
                (Object::Number(left_num), Object::Number(right_num)) => {
                    Ok(Object::Number(left_num / right_num))
                }
                _ => self.number_operand_error(operator),
            },
            TokenType::Star => match (l, r) {
                (Object::Number(left_num), Object::Number(right_num)) => {
                    Ok(Object::Number(left_num * right_num))
                }
                _ => self.number_operand_error(operator),
            },
            TokenType::Plus => match (l, r) {
                (Object::Number(left_num), Object::Number(right_num)) => {
                    Ok(Object::Number(left_num + right_num))
                }
                (Object::String(left_str), Object::String(right_str)) => {
                    Ok(Object::String(left_str.clone() + &right_str))
                }
                _ => Err(Error::Runtime {
                    token: operator.clone(),
                    message: "Operands must be two numbers or two strings".to_string(),
                }),
            },
            TokenType::GreaterEqual => match (l, r) {
                (Object::Number(left_num), Object::Number(right_num)) => {
                    Ok(Object::Boolean(left_num >= right_num))
                }
                _ => self.number_operand_error(operator),
            },
            TokenType::Greater => match (l, r) {
                (Object::Number(left_num), Object::Number(right_num)) => {
                    Ok(Object::Boolean(left_num > right_num))
                }
                _ => self.number_operand_error(operator),
            },
            TokenType::LessEqual => match (l, r) {
                (Object::Number(left_num), Object::Number(right_num)) => {
                    Ok(Object::Boolean(left_num <= right_num))
                }
                _ => self.number_operand_error(operator),
            },
            TokenType::Less => match (l, r) {
                (Object::Number(left_num), Object::Number(right_num)) => {
                    Ok(Object::Boolean(left_num < right_num))
                }
                _ => self.number_operand_error(operator),
            },
            TokenType::BangEqual => Ok(Object::Boolean(!self.is_equal(&l, &r))),
            TokenType::EqualEqual => Ok(Object::Boolean(self.is_equal(&l, &r))),
            _ => unreachable!(),
        }
    }
}
