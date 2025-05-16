# Lox-rs: A Complete Rust Implementation of the Lox Language

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
![Rust](https://img.shields.io/badge/rust-stable-orange.svg)

This project is a feature-complete interpreter for the Lox programming language, implemented in idiomatic Rust. Based on Robert Nystrom's book ["Crafting Interpreters"](https://craftinginterpreters.com/), this implementation leverages Rust's strengths to create a memory-safe, performant interpreter with clean, maintainable code.

## Features

- **Complete language implementation** including:

  - Lexical scanning and tokenization
  - Recursive descent parser with proper precedence
  - AST (Abstract Syntax Tree) based evaluation
  - Dynamic typing system
  - Variables and assignment
  - Control flow (if/else, while, for loops)
  - First-class functions with closures
  - Classes with inheritance
  - Method calls with `this` binding
  - Superclass method access with `super`
  - Block scoping with lexical environments
  - Comprehensive error reporting

- **Idiomatic Rust design** showcasing:
  - Strong type-safety through Rust's type system
  - Memory safety without garbage collection
  - Functional programming techniques
  - Advanced pattern matching
  - Zero-cost abstractions
  - Smart pointers for managing object lifetimes

## Technical Highlights

### Object System

The interpreter implements a robust dynamic object system using Rust's enum types and smart pointers:

```rust
#[derive(Debug, Clone)]
pub enum Object {
    Boolean(bool),
    Callable(Function),
    Class(Rc<RefCell<LoxClass>>),
    Instance(Rc<RefCell<LoxInstance>>),
    Null,
    Number(f64),
    String(String),
}
```

Classes and instances are managed with reference-counted cells (`Rc<RefCell<T>>`) to allow for shared ownership and interior mutability, essential for modeling object-oriented concepts in a memory-safe way.

### Memory Management

The environment uses a parent-pointer tree structure with `Rc<RefCell<T>>` for efficient variable lookup while maintaining memory safety. This approach provides:

- Proper scoping for variables
- Clean handling of nested environments
- Support for closures that capture their lexical environment
- No need for manual memory management or GC

### Visitor Pattern Implementation

The interpreter implements the visitor pattern using Rust traits and generics, providing type-safe traversal of the AST while maintaining separation of concerns between syntax and execution.

```rust
impl expr::Visitor<Object> for Interpreter {
    fn visit_binary_expr(
        &mut self,
        left: &Expr,
        operator: &Token,
        right: &Expr,
    ) -> Result<Object, Error> {
        let l = self.evaluate(left)?;
        let r = self.evaluate(right)?;

        // Type checking and operation handling
        match operator.token_type {
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
            // Other operations...
        }
    }
    // Other visitor methods...
}
```

### Lexical Resolution

The implementation includes a static analysis pass before execution:

```rust
pub fn resolve(&mut self, name: &Token, depth: usize) {
    self.locals.insert(name.clone(), depth);
}
```

This resolution step:

- Improves variable lookup performance
- Ensures proper closure semantics
- Detects errors like referencing a variable in its own initializer
- Validates proper use of `this` and `super` references

### Error Handling

Error propagation uses Rust's `Result` type with detailed error information, enabling:

- Clear error messages with source line information
- Proper error recovery in the parser
- Graceful handling of runtime errors
- Special handling for return statements via a custom Error variant

### Functional Programming Techniques

The codebase demonstrates functional programming techniques in Rust, such as:

- **Monadic Error Handling**: Using `Result` and `Option` types as monads for clean error propagation
- **Combinators**: Utilizing functions like `map`, `unwrap_or`, and other functional combinators for more expressive code
- **Declarative Style**: Favoring declarative over imperative code where appropriate
- **Immutability**: Preferring immutable data structures when possible

These approaches lead to more concise and robust code, reducing the likelihood of bugs while improving readability.

## Project Structure

- `main.rs` - Program entry point, REPL and file execution logic
- `scanner.rs` - Lexical scanner that converts source code to tokens
- `token.rs` - Token definitions and utilities
- `parser.rs` - Recursive descent parser that builds the AST
- `syntax.rs` - AST node definitions and visitor implementation
- `interpreter.rs` - Tree-walk interpreter for execution
- `environment.rs` - Variable scope and environment handling
- `resolver.rs` - Static analyzer for variable resolution
- `object.rs` - Runtime value representations
- `class.rs` - Class and instance implementations
- `function.rs` - Function call mechanism and closures
- `error.rs` - Error types and reporting
- `build.rs` - Build-time code generation for keywords

## Language Examples

### Variables and Expressions

```lox
var a = 1;
var b = 2;
print a + b; // Outputs: 3
```

### Control Flow

```lox
var a = 1;
if (a > 2)
    print "Greater than two";
else
    print "Less than two";
```

### Functions and Closures

```lox
fun makeCounter() {
  var i = 0;
  fun count() {
    i = i + 1;
    print i;
  }

  return count;
}

var counter = makeCounter();
counter(); // "1".
counter(); // "2".
```

### Classes and Inheritance

```lox
class Doughnut {
  cook() {
    print "Fry until golden brown.";
  }
}

class BostonCream < Doughnut {
  cook() {
    super.cook();
    print "Pipe full of custard and coat with chocolate.";
  }
}

BostonCream().cook();
```

### Methods and `this` Binding

```lox
class Cake {
  taste() {
    var adjective = "delicious";
    print "The " + this.flavor + " cake is " + adjective + "!";
  }
}

var cake = Cake();
cake.flavor = "German chocolate";
cake.taste(); // Prints "The German chocolate cake is delicious!".
```

## Building and Running

### Prerequisites

- Rust stable or newer
- Cargo

### Build

```bash
cargo build --release
```

### Run

Execute a Lox script:

```bash
cargo run --release -- path/to/script.lox
```

Start the REPL:

```bash
cargo run --release
```

### Test

```bash
cargo test
```

## Examples Directory

The repository includes several example Lox programs in the `examples/` directory:

```
examples/
├── assign.lox                   - Basic variable declaration and assignment
├── branching.lox                - If/else control flow
├── class.lox                    - Comprehensive class example with methods and properties
├── eat_bacon.lox                - Simple class with method call
├── fibonacci_for.lox            - Fibonacci sequence using for loops
├── fibonacci_recursive.lox      - Recursive Fibonacci implementation
├── fibonacci_while.lox          - Fibonacci sequence using while loops
├── global_block_closure_scope.lox - Demonstrates closure scope resolution
├── incorrect_super.lox          - Example of invalid super usage (for error testing)
├── inherit_method.lox           - Basic inheritance example
├── instance.lox                 - Class instantiation example
├── logical.lox                  - Logical operators with short-circuit evaluation
├── make_counter.lox             - Closure example with counter function
├── method.lox                   - Class method demonstration
├── print.lox                    - Basic printing of different types
├── scope.lox                    - Nested scope demonstration
├── super_method.lox             - Superclass method access example
└── this.lox                     - Demonstration of this binding in methods
```

To run an example:

```bash
cargo run --release -- examples/class.lox
```

## Continuous Integration

The project includes a GitHub Actions workflow that:

- Runs all tests
- Builds the release binary
- Executes the examples to verify interpreter functionality

## What I Learned

This project served as a dual learning experience, deepening my understanding of both Rust and interpreter design:

### Rust Insights

- **Ownership System**: Implementing environments and variable scoping required careful consideration of Rust's ownership rules
- **Smart Pointers**: Using `Rc<RefCell<T>>` to create parent-pointer trees for lexical scoping and object representation
- **Error Handling**: Leveraging Rust's Result type for robust error propagation throughout the interpreter
- **Pattern Matching**: Applying exhaustive pattern matching for elegant handling of AST nodes and object types
- **Traits and Generics**: Implementing the visitor pattern through Rust's trait system
- **Functional Programming**: Using combinators like `map`, `unwrap_or`, and more declarative approaches vs. imperative code

### Interpreter Concepts

- **Lexical Analysis**: Transforming raw source text into meaningful tokens
- **Recursive Descent Parsing**: Implementing a hand-written parser for grammar rules
- **Abstract Syntax Trees**: Representing code as data structures that can be traversed
- **Type Systems**: Runtime type checking and dynamic typing
- **Evaluation Strategies**: Implementing proper order of operations and short-circuit evaluation
- **Environments**: Creating a chain of environments for lexical variable lookup
- **Closures**: Capturing lexical environment for first-class functions
- **Classes and Inheritance**: Implementing an object system with method dispatch and inheritance
- **Error Recovery**: Implementing synchronization points for error recovery in the parser

## Future Enhancements

Possible extensions to this project:

- Just-in-time compilation for performance improvement
- Standard library implementation
- Module system
- Additional language features like arrays and maps
- Static type checker
- Bytecode VM implementation (similar to Part III of Crafting Interpreters)

## Acknowledgements

This project is based on Robert Nystrom's excellent book ["Crafting Interpreters"](https://craftinginterpreters.com/), which provides a clear and thorough explanation of interpreter implementation. The original Java code has been thoughtfully adapted to idiomatic Rust.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
