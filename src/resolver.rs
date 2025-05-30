use crate::error::{report, Error};
use crate::interpreter::Interpreter;
use crate::syntax::{expr, stmt};
use crate::syntax::{Expr, LiteralValue, Stmt};
use crate::token::{Token, TokenType};

use std::collections::HashMap;
use std::mem;

// Much like we track scopes as we walk the tree, we can track whether or not
// the code we are currently visiting is inside a function declaration.
#[derive(Debug, Clone)]
enum FunctionType {
    None,
    Function,
    Initializer,
    Method,
}

#[derive(Debug, Clone)]
enum ClassType {
    None,
    Class,
    SubClass,
}

pub struct Resolver<'i> {
    interpreter: &'i mut Interpreter,
    // This field keeps track of the stack of scopes currently, uh, in scope.
    // Each element in the stack is a Map representing a single block scope.
    // Keys, as in Environment, are variable names.

    // The scope stack is only used for local block scopes. Variables declared
    // at the top level in the global scope are not tracked by the resolver
    // since they are more dynamic in Lox. When resolving a variable, if we
    // can’t find it in the stack of local scopes, we assume it must be global.
    scopes: Vec<HashMap<String, bool>>,

    current_function: FunctionType,
    current_class: ClassType,

    pub had_error: bool,
}

impl<'i> Resolver<'i> {
    pub fn new(interpreter: &'i mut Interpreter) -> Self {
        Resolver {
            interpreter: interpreter,
            scopes: Vec::new(),
            current_function: FunctionType::None,
            current_class: ClassType::None,
            had_error: false,
        }
    }

    fn resolve_stmt(&mut self, statement: &Stmt) {
        let _ = statement.accept(self);
    }

    pub fn resolve_stmts(&mut self, statements: &Vec<Stmt>) {
        for statement in statements {
            self.resolve_stmt(statement)
        }
    }

    fn resolve_expr(&mut self, expression: &Expr) {
        let _ = expression.accept(self);
    }

    // A new lexical scope is created
    // Lexical scopes nest in both the interpreter and the resolver. They behave like a stack.
    // The interpreter implements that stack using a linked list - the chain of Environment objects.
    // In the resolver, we use a vector like a stack.
    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    // Declaration adds the variable to the innermost scope so that it shadows
    // any outer one and so that we know the variable exists. We mark it as “not
    // ready yet” by binding its name to false in the scope map. The value
    // associated with a key in the scope map represents whether or not we have
    // finished resolving that variable’s initializer.

    // This would help us catch errors like var a = a + 1;
    fn declare(&mut self, name: &Token) {
        let mut already_defined: bool = false;
        match self.scopes.last_mut() {
            Some(ref mut scope) => {
                already_defined = scope.contains_key(&name.lexeme);
                scope.insert(name.lexeme.clone(), false);
            }
            None => (),
        };

        // Report an error if the variable was already defined.
        if already_defined {
            self.error(
                name,
                "Variable with this name already declared in this scope.",
            );
        }
    }

    // After declaring the variable, we resolve its initializer expression in
    // that same scope where the new variable now exists but is unavailable.
    // Once the initializer expression is done, the variable is ready for prime
    // time.
    fn define(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.lexeme.clone(), true);
        }
    }

    // After that check, we actually resolve the variable itself using this helper

    // We start at the innermost scope and work outwards, looking in each map
    // for a matching name. If we find the variable, we resolve it, passing in
    // the number of scopes between the current innermost scope and the scope
    // where the variable was found. So, if the variable was found in the
    // current scope, we pass in 0.

    // If we walk through all of the block scopes and never find the variable, we leave it unresolved and assume it's global.

    fn resolve_local(&mut self, name: &Token) {
        for (i, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(&name.lexeme) {
                self.interpreter.resolve(name, i);
            }
        }
    }

    // Create a new scope for the body and then binds variables for each of the
    // function's parameters. Once that's ready, it resolve the function body in
    // that scope. This is different from how the interpreter handles function
    // declaration. At runtime, declaring a function doesn't do anything with
    // the function's body. The body doesn't get touched until later when the
    // function is called. In static analysis, we immediately traverse into the
    // body right then and there.
    fn resolve_function(&mut self, params: &Vec<Token>, body: &Vec<Stmt>, tpe: FunctionType) {
        // We stash the previous value of the field in a local variable first.
        // Remember, Lox has local functions, so you can nest function
        // declarations arbitrarily deeply. We need to track not just that we’re
        // in a function, but how many we’re in.
        let enclosing_function = self.current_function.clone();
        self.current_function = tpe;
        self.begin_scope();
        for param in params {
            self.declare(param);
            self.define(param);
        }
        self.resolve_stmts(body);
        self.end_scope();
        self.current_function = enclosing_function;
    }

    fn error(&mut self, token: &Token, message: &str) {
        if token.token_type == TokenType::Eof {
            report(token.line, " at end", message);
        } else {
            report(token.line, &format!(" at '{}'", token.lexeme), message);
        }
        self.had_error = true;
    }
}

// Only a few kinds of nodes are interesting when it comes to resolving
// variables: A block statement introduces a new scope for the statements it
// contains. A function declaration introduces a new scope for its body and
// binds its parameters in that scope. A variable declaration adds a new
// variable to the current scope. Variable and assignment expressions need to
// have their variables resolved. The rest of the nodes don’t do anything
// special, but we still need to implement visit methods for them that traverse
// into their subtrees. Even though a + expression doesn’t itself have any
// variables to resolve, either of its operands might.

impl<'i> expr::Visitor<()> for Resolver<'i> {
    fn visit_variable_expr(&mut self, name: &Token) -> Result<(), Error> {
        // First, we check to see if the variable is being accessed inside its
        // own initializer. If the variable exists in the current scope but its
        // value is false, that means we have declared it but not yet defined
        if let Some(scope) = self.scopes.last() {
            if let Some(flag) = scope.get(&name.lexeme) {
                if *flag == false {
                    self.error(name, "Cannot read local variable in its own initializer.");
                }
            }
        };
        self.resolve_local(name);
        Ok(())
    }

    // First, we resolve the expression for the assigned value in case it also
    // contains references to other variables. Then we use our existing
    // resolveLocal() method to resolve the variable that’s being assigned to.ß
    fn visit_assign_expr(&mut self, name: &Token, value: &Expr) -> Result<(), Error> {
        self.resolve_expr(value);
        self.resolve_local(name);
        Ok(())
    }

    fn visit_binary_expr(
        &mut self,
        left: &Expr,
        _operator: &Token,
        right: &Expr,
    ) -> Result<(), Error> {
        self.resolve_expr(left);
        self.resolve_expr(right);
        Ok(())
    }

    // During resolution, we recurse only into the expression to the left of the
    // dot. The actual property access happens in the interpreter.
    fn visit_get_expr(&mut self, object: &Expr, _name: &Token) -> Result<(), Error> {
        self.resolve_expr(object);
        Ok(())
    }

    // Again, like Expr.Get, the property itself is dynamically evaluated, so
    // there’s nothing to resolve there. All we need to do is recurse into the
    // two subexpressions of Expr.Set, the object whose property is being set,
    // and the value it’s being set to.
    fn visit_set_expr(&mut self, object: &Expr, _name: &Token, value: &Expr) -> Result<(), Error> {
        self.resolve_expr(value);
        self.resolve_expr(object);
        Ok(())
    }

    fn visit_super_expr(&mut self, keyword: &Token, _method: &Token) -> Result<(), Error> {
        match self.current_class {
            ClassType::None => self.error(keyword, "Cannot use 'super' outside of a class."),
            ClassType::Class => {
                self.error(keyword, "Cannot use 'super' in a class with no superclass.")
            }
            _ => self.resolve_local(keyword),
        }
        Ok(())
    }

    fn visit_this_expr(&mut self, keyword: &Token) -> Result<(), Error> {
        if let ClassType::None = self.current_class {
            self.error(keyword, "Cannot use 'this' outside of a class.");
        } else {
            self.resolve_local(keyword);
        }
        Ok(())
    }

    // We walk the argument list and resolve them all. The thing being called is
    // also an expression (usually a variable expression), so that gets resolved
    // too.

    // property dispatch in Lox is dynamic since we don’t process the property
    // name during the static resolution pass.
    fn visit_call_expr(
        &mut self,
        callee: &Expr,
        _paren: &Token,
        arguments: &Vec<Expr>,
    ) -> Result<(), Error> {
        self.resolve_expr(callee);
        for argument in arguments {
            self.resolve_expr(argument);
        }
        Ok(())
    }

    fn visit_grouping_expr(&mut self, expression: &Expr) -> Result<(), Error> {
        self.resolve_expr(expression);
        Ok(())
    }

    fn visit_literal_expr(&self, _value: &LiteralValue) -> Result<(), Error> {
        Ok(())
    }

    // Since a static analysis does no control flow or short-circuiting, logical expressions are exactly the same as other binary operators
    fn visit_logical_expr(
        &mut self,
        left: &Expr,
        _operator: &Token,
        right: &Expr,
    ) -> Result<(), Error> {
        self.resolve_expr(left);
        self.resolve_expr(right);
        Ok(())
    }

    fn visit_unary_expr(&mut self, _operator: &Token, right: &Expr) -> Result<(), Error> {
        self.resolve_expr(right);
        Ok(())
    }
}

impl<'i> stmt::Visitor<()> for Resolver<'i> {
    fn visit_block_stmt(&mut self, statements: &Vec<Stmt>) -> Result<(), Error> {
        self.begin_scope();
        self.resolve_stmts(statements);
        self.end_scope();
        Ok(())
    }

    // whenever a this expression is encountered (at least inside a method) it
    // will resolve to a “local variable” defined in an implicit scope just
    // outside of the block for the method body.
    fn visit_class_stmt(
        &mut self,
        name: &Token,
        superclass: &Option<Expr>,
        methods: &Vec<Stmt>,
    ) -> Result<(), Error> {
        let enclosing_class = mem::replace(&mut self.current_class, ClassType::Class);

        self.declare(name);
        self.define(name);

        if let Some(Expr::Variable {
            name: superclass_name,
        }) = superclass
        {
            if name.lexeme == superclass_name.lexeme {
                self.error(superclass_name, "A class cannot inherit from itself.")
            }

            self.current_class = ClassType::SubClass;
            self.resolve_local(superclass_name);

            self.begin_scope();
            self.scopes
                .last_mut()
                .expect("Scopes is empty.")
                .insert("super".to_owned(), true);
        }

        self.begin_scope();
        self.scopes
            .last_mut()
            .expect("Scopes is empty.")
            .insert("this".to_owned(), true);

        for method in methods {
            if let Stmt::Function { name, params, body } = method {
                let declaration = if name.lexeme == "init" {
                    FunctionType::Initializer
                } else {
                    FunctionType::Method
                };
                self.resolve_function(params, body, declaration);
            } else {
                unreachable!()
            }
        }

        if superclass.is_some() {
            self.end_scope()
        }

        self.end_scope();

        self.current_class = enclosing_class;

        Ok(())
    }

    // An expression statement contains a single expression to traverse.
    fn visit_expression_stmt(&mut self, expression: &Expr) -> Result<(), Error> {
        self.resolve_expr(expression);
        Ok(())
    }

    // An if statement has an expression for its condition and one or two statements for the branches.
    fn visit_if_stmt(
        &mut self,
        condition: &Expr,
        then_branch: &Stmt,
        else_branch: &Option<Stmt>,
    ) -> Result<(), Error> {
        self.resolve_expr(condition);
        self.resolve_stmt(then_branch);
        if let Some(else_stmt) = else_branch {
            self.resolve_stmt(else_stmt);
        }
        Ok(())
    }

    fn visit_print_stmt(&mut self, expression: &Expr) -> Result<(), Error> {
        self.resolve_expr(expression);
        Ok(())
    }

    fn visit_return_stmt(&mut self, keyword: &Token, value: &Option<Expr>) -> Result<(), Error> {
        if let FunctionType::None = self.current_function {
            self.error(keyword, "Cannot return from top-level code.");
        }

        if let Some(return_value) = value {
            if let FunctionType::Initializer = self.current_function {
                self.error(keyword, "Can't return a value from an initializer.");
            }
            self.resolve_expr(return_value);
        }
        Ok(())
    }

    // We resolve its condition and resolve the body exactly once
    fn visit_while_stmt(&mut self, condition: &Expr, body: &Stmt) -> Result<(), Error> {
        self.resolve_expr(condition);
        self.resolve_stmt(body);
        Ok(())
    }

    // We split binding into two steps, declaring then defining, in order to handle funny edge cases like this:
    /*
    var a = "outer";
    {
      var a = a;
    }
    */
    fn visit_var_stmt(&mut self, name: &Token, initializer: &Option<Expr>) -> Result<(), Error> {
        self.declare(name);
        if let Some(init) = initializer {
            self.resolve_expr(init);
        }
        self.define(name);
        Ok(())
    }

    // Similar to visit_variable_stmt(), we declare and define the name of the
    // function in the current scope. Unlike variables, though, we define the
    // name eagerly, before resolving the function's body. This lets a function
    // recursively refer to itself inside its own body.
    fn visit_function_stmt(
        &mut self,
        name: &Token,
        params: &Vec<Token>,
        body: &Vec<Stmt>,
    ) -> Result<(), Error> {
        self.declare(name);
        self.define(name);

        self.resolve_function(params, body, FunctionType::Function);
        Ok(())
    }
}
