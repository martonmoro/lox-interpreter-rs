use crate::environment::{self, Environment};
use crate::error::Error;
use crate::interpreter::{self, Interpreter};
use crate::object::Object;
use crate::syntax::Stmt;
use crate::token::Token;

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

#[derive(Clone)]
pub enum Function {
    // These are functions that the interpreter exposes to user code but that
    // are implemented in the host language. Sometimes these are called
    // primitives, external functions, or foreign functions. (after this we
    // could simplify Lox and replace the built-in print statement) Many
    // languages also allow users to provide their own native functions. The
    // mechanism for doing so is called a foreign function interface (FFI),
    // native extension, native interface, or something along those lines. Toß
    // add a native function, the book uses anonymous class instances that
    // implement the LoxCallable interface.
    Native {
        arity: usize,
        body: Box<fn(&Vec<Object>) -> Object>,
    },

    // LoxFunction in the book
    User {
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
        closure: Rc<RefCell<Environment>>,
    },
}

impl Function {
    // We pass in the interpreter in case the class implementing
    // call() needs it. We also give it the list of evaluated
    // argument values. The implementer’s job is then to return the
    // value that the call expression produces.
    pub fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: &Vec<Object>,
    ) -> Result<Object, Error> {
        match self {
            Function::Native { body, .. } => Ok(body(arguments)),
            Function::User {
                params,
                body,
                closure,
                ..
            } => {
                // This means each function gets its own environment where it stores those variables.

                // Further, this environment must be created dynamically. Each
                // function call gets its own environment. Otherwise, recursion
                // would break. If there are multiple calls to the same function
                // in play at the same time, each needs its own environment,
                // even though they are all calls to the same function.
                let mut environment = Rc::new(RefCell::new(Environment::from(closure)));
                for (param, argument) in params.iter().zip(arguments.iter()) {
                    environment
                        .borrow_mut()
                        .define(param.lexeme.clone(), argument.clone());
                }
                match interpreter.execute_block(body, environment) {
                    Err(Error::Return { value }) => Ok(value),
                    Err(other) => Err(other),
                    Ok(..) => Ok(Object::Null), // We don't have a return statement
                }
            }
        }
    }

    // We create a new environment nestled inside the method’s original closure.
    // Sort of a closure-within-a-closure. When the method is called, that will
    // become the parent of the method body’s environment. We declare “this” as
    // a variable in that environment and bind it to the given instance, the
    // instance that the method is being accessed from.ß
    pub fn bind(&self, instance: Object) -> Self {
        match self {
            Function::Native { .. } => unreachable!(),
            Function::User {
                name,
                params,
                body,
                closure,
            } => {
                let environment = Rc::new(RefCell::new(Environment::from(closure)));
                environment
                    .borrow_mut()
                    .define("this".to_string(), instance);
                Function::User {
                    name: name.clone(),
                    params: params.clone(),
                    body: body.clone(),
                    closure: environment,
                }
            }
        }
    }

    pub fn arity(&self) -> usize {
        match self {
            Function::Native { arity, .. } => *arity,
            Function::User { params, .. } => params.len(),
        }
    }
}

// Implements to_string which corresponds to toString from the book
impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Function::Native { .. } => write!(f, "<native func>"),
            Function::User { name, .. } => write!(f, "<fn {}>", name.lexeme),
        }
    }
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Function::Native { .. } => write!(f, "<native func>"),
            Function::User { name, .. } => write!(f, "<fn {}>", name.lexeme),
        }
    }
}
