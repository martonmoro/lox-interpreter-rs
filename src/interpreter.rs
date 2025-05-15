use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::class::{LoxClass, LoxInstance};
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
    // side table: tabular data structure that stores data separately from the
    // objects it relates to Interactive tools like IDEs often incrementally
    // reparse and re-resolve parts of the user’s program. It may be hard to
    // find all of the bits of state that need recalculating when they’re hiding
    // in the foliage of the syntax tree. A benefit of storing this data outside
    // of the nodes is that it makes it easy to discard it—simply clear the map.
    locals: HashMap<Token, usize>,
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
            environment: Rc::clone(&globals),
            locals: HashMap::new(),
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

    // Each time it visits a variable, it tells the interpreter how many scopes
    // there are between the current scope and the scope where the variable is
    // defined. At runtime, this corresponds exactly to the number of
    // environments between the current one and the enclosing one where the
    // interpreter can find the variable’s value.
    pub fn resolve(&mut self, name: &Token, depth: usize) {
        // We want to store the resolution information somewhere so we can use
        // it when the variable or assignment expression is later executed, but
        // where? One obvious place is right in the syntax tree node itself.
        // That’s a fine approach, and that’s where many compilers store the
        // results of analyses like this. But instead, we’ll take another common
        // approach and store it off to the side in a map that associates each
        // syntax tree node with its resolved data.
        self.locals.insert(name.clone(), depth);
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
            Object::Class(class) => class.borrow().name.clone(),
            Object::Instance(instance) => {
                format!("{} instance", instance.borrow().class.borrow().name)
            }
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

    // First, we look up the resolved distance in the map. Remember that we
    // resolved only local variables. Globals are treated specially and don't
    // end up in the map. So, if we don't find a distance in the map, it must be
    // global. In that case, we look it up dynamically, directly in the global
    // env. That throws a runtime error if the variable isn't defined.

    // If we do get a distance, we have a local variable, and we get to take
    // advantage of the results of our static analysis. Instead of calling
    // get(), we call this new method on Environment.
    fn look_up_variable(&self, name: &Token) -> Result<Object, Error> {
        if let Some(distance) = self.locals.get(name) {
            self.environment.borrow().get_at(*distance, name)
        } else {
            self.globals.borrow().get(name)
        }
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

        match callee_value {
            Object::Callable(function) => {
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
            }
            Object::Class(ref class) => {
                // This is the call method of a class.
                let args_size = args.len();
                let instance = LoxInstance::new(class);
                if let Some(initializer) = class.borrow().find_method("init") {
                    if args_size != initializer.arity() {
                        return Err(Error::Runtime {
                            token: paren.clone(),
                            message: format!(
                                "Expected {} arguments but got {}.",
                                initializer.arity(),
                                args_size
                            ),
                        });
                    } else {
                        initializer.bind(instance.clone()).call(self, &args)?;
                    }
                }

                Ok(instance)
            }
            _ => Err(Error::Runtime {
                token: paren.clone(),
                message: "Can only call functions and classes.".to_string(),
            }),
        }
    }

    // First, we evaluate the expression whose property is being accessed. In
    // Lox, only instances of classes have properties. If the object is some
    // other type like a number, invoking a getter on it is a runtime error.
    fn visit_get_expr(&mut self, object: &Expr, name: &Token) -> Result<Object, Error> {
        let object = self.evaluate(object)?;
        if let Object::Instance(ref instance) = object {
            instance.borrow().get(name, &object)
        } else {
            Err(Error::Runtime {
                token: name.clone(),
                message: "Only instances have properties.".to_string(),
            })
        }
    }

    // We evaluate the object whose property is being set and check to see if
    // it’s a LoxInstance. If not, that’s a runtime error. Otherwise, we
    // evaluate the value being set and store it on the instance.
    fn visit_set_expr(
        &mut self,
        object: &Expr,
        property_name: &Token,
        value: &Expr,
    ) -> Result<Object, Error> {
        let object = self.evaluate(object)?;
        if let Object::Instance(ref instance) = object {
            let value = self.evaluate(value)?;
            instance.borrow_mut().set(property_name, value);
            let r = Object::Instance(Rc::clone(instance));
            Ok(r)
        } else {
            Err(Error::Runtime {
                token: property_name.clone(),
                message: "Only instances have fields.".to_string(),
            })
        }
    }

    fn visit_this_expr(&mut self, keyword: &Token) -> Result<Object, Error> {
        self.look_up_variable(keyword)
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
        self.look_up_variable(name)
    }

    fn visit_assign_expr(&mut self, name: &Token, value: &Expr) -> Result<Object, Error> {
        let v = self.evaluate(value)?;
        if let Some(distance) = self.locals.get(name) {
            self.environment
                .borrow_mut()
                .assign_at(*distance, name, v.clone())?;
        } else {
            // TODO: globals or environment?
            self.globals.borrow_mut().assign(name, v.clone())?;
        }
        Ok(v)
    }
}

impl stmt::Visitor<()> for Interpreter {
    fn visit_expression_stmt(&mut self, expression: &Expr) -> Result<(), Error> {
        self.evaluate(expression)?;
        Ok(())
    }

    // We declare the class's name in the current environment. Then we turn the
    // class syntax node into a LoxClass, the runtime representation of a class.
    // We circle back and store the class object in the variable we previously
    // declared. That two-stage variable binding process allows references to
    // the class inside its own methods.
    fn visit_class_stmt(&mut self, class_name: &Token, methods: &Vec<Stmt>) -> Result<(), Error> {
        self.environment
            .borrow_mut()
            .define(class_name.lexeme.clone(), Object::Null);

        // When we interpret a class declaration statement, we turn the
        // syntactic representation of the class—its AST node—into its runtime
        // representation. Now, we need to do that for the methods contained in
        // the class as well. Each method declaration blossoms into a
        // LoxFunction object.
        let mut class_methods: HashMap<String, Function> = HashMap::new();
        for method in methods {
            if let Stmt::Function { name, params, body } = method {
                let function = Function::User {
                    name: name.clone(),
                    params: params.clone(),
                    body: body.clone(),
                    closure: Rc::clone(&self.environment),
                    is_initializer: name.lexeme == "init",
                };
                class_methods.insert(name.lexeme.clone(), function);
            } else {
                unreachable!()
            }
        }

        let lox_class = LoxClass {
            name: class_name.lexeme.clone(),
            methods: class_methods,
        };
        let class = Object::Class(Rc::new(RefCell::new(lox_class)));
        self.environment.borrow_mut().assign(class_name, class)?;
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
            is_initializer: false,
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
