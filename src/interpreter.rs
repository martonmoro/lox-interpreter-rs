use std::cell::RefCell;
use std::rc::Rc;

use crate::environment::Environment;
use crate::error::Error;
use crate::object::Object;
use crate::syntax::{expr, stmt, Stmt};
use crate::syntax::{Expr, LiteralValue};
use crate::token::{Token, TokenType};
pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Rc::new(RefCell::new(Environment::new())),
        }
    }

    pub fn interpret(&self, statements: &Vec<Stmt>) -> Result<(), Error> {
        for statement in statements {
            self.execute(statement)?;
        }
        Ok(())
    }

    fn execute(&self, stmt: &Stmt) -> Result<(), Error> {
        stmt.accept(self)
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

impl expr::Visitor<Object> for Interpreter {
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

    fn visit_variable_expr(&self, name: &Token) -> Result<Object, Error> {
        self.environment.borrow().get(name)
    }

    fn visit_assign_expr(&self, name: &Token, value: &Expr) -> Result<Object, Error> {
        let v = self.evaluate(value)?;
        self.environment.borrow_mut().assign(name, v.clone())?;
        Ok(v)
    }
}

impl stmt::Visitor<()> for Interpreter {
    fn visit_expression_stmt(&self, expression: &Expr) -> Result<(), Error> {
        self.evaluate(expression)?;
        Ok(())
    }
    fn visit_print_stmt(&self, expression: &Expr) -> Result<(), Error> {
        let value = self.evaluate(expression)?;
        println!("{}", self.stringify(value));
        Ok(())
    }
    // if we strictly wanted to follow the book we could do
    // fn visit_var_stmt(&self, name: &Token, initializer: &Option<Expr>) -> Result<(), Error> {
    //     let value = if let Some(initializer) = initializer {
    //         self.evaluate(initializer)?
    //     } else {
    //         Object::Null
    //     };

    //     self.environment
    //         .borrow_mut()
    //         .define(name.lexeme.clone(), value);

    //     Ok(())
    // }

    // if we want to do more functional style
    fn visit_var_stmt(&self, name: &Token, initializer: &Option<Expr>) -> Result<(), Error> {
        let value = initializer
            .as_ref() // we want to borrow the Expr
            .map(|i| self.evaluate(i)) // if it was a some call self.evaluate and wrap the result in a Some, if None leave it as None
            .unwrap_or(Ok(Object::Null))?; // unwrap result or return Ok(Object::Null)

        self.environment
            .borrow_mut()
            .define(name.lexeme.clone(), value);

        Ok(())
    }
}
