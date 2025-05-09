use std::cell::RefCell;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::environment::Environment;
use crate::error::Error;
use crate::function::Function;
use crate::object::Object;
use crate::syntax::{expr, stmt, Stmt};
use crate::syntax::{Expr, LiteralValue};
use crate::token::{Token, TokenType};
pub struct Interpreter {
    // Fix reference to the outermost global env
    pub globals: Rc<RefCell<Environment>>,
    environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = Rc::new(RefCell::new(Environment::new()));
        let clock: Object = Object::Callable(Function::Native {
            arity: 0,
            body: Box::new(|_args: &Vec<Object>| {
                Object::Number(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("Could not retrieve time.")
                        .as_millis() as f64,
                )
            }),
        });
        // In Lox functions and variables occupy the same namespace.
        globals.borrow_mut().define("clock".to_string(), clock);
        Self {
            globals: Rc::clone(&globals),
            environment: Rc::new(RefCell::new(Environment::new())),
        }
    }

    pub fn interpret(&mut self, statements: &Vec<Stmt>) -> Result<(), Error> {
        for statement in statements {
            self.execute(statement)?;
        }
        Ok(())
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<(), Error> {
        stmt.accept(self)
    }

    /*
    Another classic approach is to explicitly pass the environment as a parameter to each visit method.
    To “change” the environment, you pass a different one as you recurse down the tree.
    You don’t have to restore the old one, since the new one lives on the Java stack and is implicitly discarded when the interpreter returns from the block’s visit method.
     */
    pub fn execute_block(
        &mut self,
        statements: &Vec<Stmt>,
        environment: Rc<RefCell<Environment>>,
    ) -> Result<(), Error> {
        let previous = self.environment.clone();

        self.environment = environment;

        let result = statements
            .iter()
            .try_for_each(|statement| self.execute(statement));

        self.environment = previous;

        result
    }

    // simply call interpreters visitor implementation
    fn evaluate(&mut self, expr: &Expr) -> Result<Object, Error> {
        expr.accept(self)
    }

    fn stringify(&self, object: Object) -> String {
        match object {
            Object::Null => "nil".to_string(),
            Object::Number(n) => n.to_string(),
            Object::Boolean(b) => b.to_string(),
            Object::String(s) => s,
            Object::Callable(f) => f.to_string(),
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

    fn visit_grouping_expr(&mut self, expression: &Expr) -> Result<Object, Error> {
        self.evaluate(expression)
    }

    fn visit_unary_expr(&mut self, operator: &Token, right: &Expr) -> Result<Object, Error> {
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

    fn visit_call_expr(
        &mut self,
        callee: &Expr,
        paren: &Token,
        arguments: &Vec<Expr>,
    ) -> Result<Object, Error> {
        let callee_value = self.evaluate(callee)?;

        let argument_values: Result<Vec<Object>, Error> = arguments
            .into_iter()
            .map(|expr| self.evaluate(expr))
            .collect();

        let args = argument_values?;

        if let Object::Callable(function) = callee_value {
            // Different languages take different approaches to this problem. Of
            // course, most statically typed languages check this at compile
            // time and refuse to compile the code if the argument count doesn’t
            // match the function’s arity. JavaScript discards any extra
            // arguments you pass. If you don’t pass enough, it fills in the
            // missing parameters with the magic
            // sort-of-like-null-but-not-really value undefined. Python is
            // stricter. It raises a runtime error if the argument list is too
            // short or too long.

            // Before invoking the callable, we check to see if the argument list’s length matches the callable’s arity.
            let args_size = args.len();
            if args_size != function.arity() {
                Err(Error::Runtime {
                    token: paren.clone(),
                    message: format!(
                        "Expected {} arguments but got {}.",
                        function.arity(),
                        args_size
                    ),
                })
            } else {
                function.call(self, &args)
            }
        } else {
            Err(Error::Runtime {
                token: paren.clone(),
                message: "Can only call functions and classes.".to_string(),
            })
        }
    }

    fn visit_binary_expr(
        &mut self,
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

    /*
       Since Lox is dynamically typed, we allow operands of any type and use truthiness to determine what each operand represents.
       We apply similar reasoning to the result.
       Instead of promising to literally return true or false, a logic operator merely guarantees it will return a value with appropriate truthiness.
    */
    fn visit_logical_expr(
        &mut self,
        left: &Expr,
        operator: &Token,
        right: &Expr,
    ) -> Result<Object, Error> {
        let l = self.evaluate(left)?;

        if operator.token_type == TokenType::Or {
            if self.is_truthy(&l) {
                return Ok(l);
            }
        } else {
            if !self.is_truthy(&l) {
                return Ok(l);
            }
        }

        self.evaluate(right)
    }

    fn visit_variable_expr(&mut self, name: &Token) -> Result<Object, Error> {
        self.environment.borrow().get(name)
    }

    fn visit_assign_expr(&mut self, name: &Token, value: &Expr) -> Result<Object, Error> {
        let v = self.evaluate(value)?;
        self.environment.borrow_mut().assign(name, v.clone())?;
        Ok(v)
    }
}

impl stmt::Visitor<()> for Interpreter {
    fn visit_expression_stmt(&mut self, expression: &Expr) -> Result<(), Error> {
        self.evaluate(expression)?;
        Ok(())
    }

    // We take a syntax node - a compile-time representation of the function - and convert it to its runtime representation
    // Function declarations are different from other literal nodes in that the declaration also binds the resulting object to a new variable
    fn visit_function_stmt(
        &mut self,
        name: &Token,
        params: &Vec<Token>,
        body: &Vec<Stmt>,
    ) -> Result<(), Error> {
        let function = Function::User {
            name: name.clone(),
            params: params.clone(),
            body: body.clone(),
            closure: Rc::clone(&self.environment),
        };
        self.environment
            .borrow_mut()
            .define(name.lexeme.clone(), Object::Callable(function));
        Ok(())
    }

    fn visit_return_stmt(&mut self, keyword: &Token, value: &Option<Expr>) -> Result<(), Error> {
        let return_value = value
            .as_ref()
            .map(|v| self.evaluate(v))
            .unwrap_or(Ok(Object::Null))?;

        // Use Err to jump back to the top of the stack
        Err(Error::Return {
            value: return_value,
        })
    }

    fn visit_if_stmt(
        &mut self,
        condition: &Expr,
        then_branch: &Stmt,
        else_branch: &Option<Stmt>,
    ) -> Result<(), Error> {
        let condition_val = self.evaluate(condition)?;
        if self.is_truthy(&condition_val) {
            self.execute(then_branch)?;
        } else if let Some(else_bran) = else_branch {
            self.execute(else_bran)?;
        }

        Ok(())
    }

    fn visit_while_stmt(&mut self, condition: &Expr, body: &Stmt) -> Result<(), Error> {
        let mut value = self.evaluate(condition)?;
        while self.is_truthy(&value) {
            self.execute(body)?;
            value = self.evaluate(condition)?
        }

        Ok(())
    }

    fn visit_print_stmt(&mut self, expression: &Expr) -> Result<(), Error> {
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
    fn visit_var_stmt(&mut self, name: &Token, initializer: &Option<Expr>) -> Result<(), Error> {
        let value = initializer
            .as_ref() // we want to borrow the Expr
            .map(|i| self.evaluate(i)) // if it was a some call self.evaluate and wrap the result in a Some, if None leave it as None
            .unwrap_or(Ok(Object::Null))?; // unwrap result or return Ok(Object::Null)

        self.environment
            .borrow_mut()
            .define(name.lexeme.clone(), value);

        Ok(())
    }

    fn visit_block_stmt(&mut self, statements: &Vec<Stmt>) -> Result<(), Error> {
        self.execute_block(
            statements,
            Rc::new(RefCell::new(Environment::from(&self.environment))),
        )?;
        Ok(())
    }
}
