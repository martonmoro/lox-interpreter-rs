use std::fmt;

use crate::error::Error;
use crate::token::Token;

// we don't really need to generate these like they are generated using a script in the book
#[derive(Debug, Clone)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        paren: Token, // We are using this token's location when we report a runtime error caused by a function call (closing paren)
        arguments: Vec<Expr>,
    },
    Get {
        object: Box<Expr>,
        name: Token,
    },
    // we are using this instead of Binary to short-circuit
    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Set {
        object: Box<Expr>,
        name: Token,
        value: Box<Expr>,
    },
    Super {
        keyword: Token,
        method: Token,
    },
    This {
        keyword: Token,
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

#[derive(Debug, Clone)]
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
    pub fn accept<R, T: expr::Visitor<R>>(&self, visitor: &mut T) -> Result<R, Error> {
        match self {
            Expr::Binary {
                left,
                operator,
                right,
            } => visitor.visit_binary_expr(left, operator, right),
            Expr::Call {
                callee,
                paren,
                arguments,
            } => visitor.visit_call_expr(callee, paren, arguments),
            Expr::Get { object, name } => visitor.visit_get_expr(object, name),
            Expr::Logical {
                left,
                operator,
                right,
            } => visitor.visit_logical_expr(left, operator, right),
            Expr::Set {
                object,
                name,
                value,
            } => visitor.visit_set_expr(object, name, value),
            Expr::Super { keyword, method } => visitor.visit_super_expr(keyword, method),
            Expr::This { keyword } => visitor.visit_this_expr(keyword),
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
            &mut self,
            left: &Expr,
            operator: &Token,
            right: &Expr,
        ) -> Result<R, Error>;
        fn visit_call_expr(
            &mut self,
            callee: &Expr,
            paren: &Token,
            arguments: &Vec<Expr>,
        ) -> Result<R, Error>;
        fn visit_get_expr(&mut self, object: &Expr, name: &Token) -> Result<R, Error>;
        fn visit_set_expr(&mut self, object: &Expr, name: &Token, value: &Expr)
            -> Result<R, Error>;
        fn visit_super_expr(&mut self, keyword: &Token, method: &Token) -> Result<R, Error>;
        fn visit_this_expr(&mut self, keyword: &Token) -> Result<R, Error>;
        fn visit_logical_expr(
            &mut self,
            left: &Expr,
            operator: &Token,
            right: &Expr,
        ) -> Result<R, Error>;
        fn visit_grouping_expr(&mut self, expression: &Expr) -> Result<R, Error>;
        fn visit_literal_expr(&self, value: &LiteralValue) -> Result<R, Error>;
        fn visit_unary_expr(&mut self, operator: &Token, right: &Expr) -> Result<R, Error>;
        fn visit_variable_expr(&mut self, name: &Token) -> Result<R, Error>;
        fn visit_assign_expr(&mut self, name: &Token, value: &Expr) -> Result<R, Error>;
    }
}
#[derive(Debug, Clone)]
pub enum Stmt {
    Block {
        statements: Vec<Stmt>,
    },
    Class {
        name: Token,
        // The grammar restricts the superclass clause to a single identifier,
        // but at runtime, that identifier is evaluated as a variable access.
        // Wrapping the name in an Expr.Variable early on in the parser gives us
        // an object that the resolver can hang the resolution information off
        // of.

        // Assuming Expr::Variable
        superclass: Option<Expr>,
        // Assuming all are Stmt::Function
        methods: Vec<Stmt>,
    },
    Expression {
        expression: Expr,
    },
    Function {
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
    },
    Return {
        keyword: Token,
        value: Option<Expr>,
    },
    Print {
        expression: Expr,
    },
    Var {
        name: Token,
        initializer: Option<Expr>,
    },
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Box<Option<Stmt>>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    Null, // placeholder until statement handling is figured out after synchronize()
}

impl Stmt {
    pub fn accept<R, T: stmt::Visitor<R>>(&self, visitor: &mut T) -> Result<R, Error> {
        match self {
            Stmt::Expression { expression } => visitor.visit_expression_stmt(expression),
            Stmt::Print { expression } => visitor.visit_print_stmt(expression),
            Stmt::Function { name, params, body } => {
                visitor.visit_function_stmt(name, params, body)
            }
            Stmt::Return { keyword, value } => visitor.visit_return_stmt(keyword, value),
            Stmt::Var { name, initializer } => visitor.visit_var_stmt(name, initializer),
            Stmt::Block { statements } => visitor.visit_block_stmt(statements),
            Stmt::Class {
                name,
                superclass,
                methods,
            } => visitor.visit_class_stmt(name, superclass, methods),
            Stmt::Null => unimplemented!(),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => visitor.visit_if_stmt(condition, then_branch, else_branch),
            Stmt::While { condition, body } => visitor.visit_while_stmt(condition, body),
        }
    }
}

pub mod stmt {
    use crate::error::Error;
    use crate::token::Token;

    use super::{Expr, Stmt};

    pub trait Visitor<R> {
        fn visit_expression_stmt(&mut self, stmt: &Expr) -> Result<R, Error>;
        fn visit_print_stmt(&mut self, stmt: &Expr) -> Result<R, Error>;
        fn visit_function_stmt(
            &mut self,
            name: &Token,
            params: &Vec<Token>,
            body: &Vec<Stmt>,
        ) -> Result<R, Error>;
        fn visit_return_stmt(&mut self, keyword: &Token, value: &Option<Expr>) -> Result<R, Error>;
        fn visit_var_stmt(&mut self, name: &Token, initializer: &Option<Expr>) -> Result<R, Error>;
        fn visit_block_stmt(&mut self, statements: &Vec<Stmt>) -> Result<R, Error>;
        fn visit_class_stmt(
            &mut self,
            name: &Token,
            superclass: &Option<Expr>,
            methods: &Vec<Stmt>,
        ) -> Result<R, Error>;
        fn visit_if_stmt(
            &mut self,
            condition: &Expr,
            then_branch: &Stmt,
            else_branch: &Option<Stmt>,
        ) -> Result<R, Error>;
        fn visit_while_stmt(&mut self, condition: &Expr, body: &Stmt) -> Result<R, Error>;
    }
}

pub struct AstPrinter;

impl AstPrinter {
    fn parenthesize(&mut self, name: String, exprs: Vec<&Expr>) -> Result<String, Error> {
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
        &mut self,
        left: &Expr,
        operator: &Token,
        right: &Expr,
    ) -> Result<String, Error> {
        self.parenthesize(operator.lexeme.clone(), vec![left, right])
    }

    fn visit_set_expr(
        &mut self,
        object: &Expr,
        name: &Token,
        value: &Expr,
    ) -> Result<String, Error> {
        self.parenthesize(name.lexeme.clone(), vec![object, value])
    }

    fn visit_super_expr(&mut self, _keyword: &Token, _method: &Token) -> Result<String, Error> {
        Ok("super".to_string())
    }

    fn visit_this_expr(&mut self, _keyword: &Token) -> Result<String, Error> {
        Ok("this".to_string())
    }

    fn visit_get_expr(&mut self, object: &Expr, name: &Token) -> Result<String, Error> {
        self.parenthesize(name.lexeme.clone(), vec![object])
    }

    fn visit_grouping_expr(&mut self, expression: &Expr) -> Result<String, Error> {
        self.parenthesize("group".to_string(), vec![expression])
    }

    fn visit_literal_expr(&self, value: &LiteralValue) -> Result<String, Error> {
        Ok(value.to_string())
    }

    fn visit_unary_expr(&mut self, operator: &Token, right: &Expr) -> Result<String, Error> {
        self.parenthesize(operator.lexeme.clone(), vec![right])
    }

    fn visit_variable_expr(&mut self, name: &Token) -> Result<String, Error> {
        Ok(name.lexeme.clone())
    }

    fn visit_assign_expr(&mut self, name: &Token, value: &Expr) -> Result<String, Error> {
        self.parenthesize(name.lexeme.clone(), vec![value])
    }

    fn visit_logical_expr(
        &mut self,
        left: &Expr,
        operator: &Token,
        right: &Expr,
    ) -> Result<String, Error> {
        self.parenthesize(operator.lexeme.clone(), vec![left, right])
    }

    fn visit_call_expr(
        &mut self,
        _callee: &Expr,
        _paren: &Token,
        _arguments: &Vec<Expr>,
    ) -> Result<String, Error> {
        unimplemented!()
    }
}
