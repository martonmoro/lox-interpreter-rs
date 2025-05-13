# Lox-rs: A Rust Implementation of the Lox Language

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
![Rust](https://img.shields.io/badge/rust-1.50%2B-orange.svg)

This project is a feature-complete interpreter for the Lox programming language, implemented in idiomatic Rust. Based on Robert Nystrom's book ["Crafting Interpreters"](https://craftinginterpreters.com/), this implementation leverages Rust's strengths to create a memory-safe, performant interpreter with clean, maintainable code.

## Features

- **Complete language implementation** including:

  - Lexical scanning and tokenization
  - Recursive descent parser with proper precedence
  - AST (Abstract Syntax Tree) based evaluation
  - Dynamic typing system
  - Variables and assignment
  - Control flow (if/else, while, for loops)
  - Block scoping with lexical environments
  - Comprehensive error reporting

- **Idiomatic Rust design** showcasing:
  - Strong type-safety through Rust's type system
  - Memory safety without garbage collection
  - Functional programming techniques
  - Advanced pattern matching
  - Zero-cost abstractions

## Technical Highlights

### Memory Management

The environment uses a parent-pointer tree structure with `Rc<RefCell<T>>` for efficient variable lookup while maintaining memory safety. This approach provides:

- Proper scoping for variables
- Clean handling of nested environments
- No need for manual memory management or GC

### Visitor Pattern Implementation

The interpreter implements the visitor pattern using Rust traits and generics, providing type-safe traversal of the AST while maintaining separation of concerns between syntax and execution.

```rust
impl expr::Visitor<Object> for Interpreter {
    fn visit_binary_expr(
        &self,
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

### Error Handling

Error propagation uses Rust's `Result` type with detailed error information, enabling:

- Clear error messages with source line information
- Proper error recovery in the parser
- Graceful handling of runtime errors

### Functional Programming Techniques

The codebase demonstrates functional programming techniques in Rust, such as:

- **Monadic Error Handling**: Using `Result` and `Option` types as monads for clean error propagation
- **Combinators**: Utilizing functions like `map`, `unwrap_or`, and other functional combinators for more expressive code
- **Declarative Style**: Favoring declarative over imperative code where appropriate
- **Immutability**: Preferring immutable data structures when possible

These approaches lead to more concise and robust code, reducing the likelihood of bugs while improving readability.

### Project Structure

- `main.rs` - Program entry point, REPL and file execution logic
- `scanner.rs` - Lexical scanner that converts source code to tokens
- `token.rs` - Token definitions and utilities
- `parser.rs` - Recursive descent parser that builds the AST
- `syntax.rs` - AST node definitions and visitor implementation
- `interpreter.rs` - Tree-walk interpreter for execution
- `environment.rs` - Variable scope and environment handling
- `object.rs` - Runtime value representations
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
// Calculate Fibonacci numbers
var a = 0;
var b = 1;

for (var i = 0; i < 10; i = i + 1) {
  print a;
  var temp = a;
  a = b;
  b = temp + b;
}
```

### Nested Scopes

```lox
var a = "global";
{
  var b = "middle";
  {
    var c = "inner";
    print a; // Prints: global
    print b; // Prints: middle
    print c; // Prints: inner
  }
  print a; // Prints: global
  print b; // Prints: middle
  // print c; // Error: undefined variable
}
```

### Functions

```lox
fun fib(n) {
  if (n <= 1) return n;
  return fib(n - 2) + fib(n - 1);
}

for (var i = 0; i < 20; i = i + 1) {
  print fib(i);
}
```

## Building and Running

### Prerequisites

- Rust 1.50 or higher
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

## Current Status

- ✅ Lexical scanning
- ✅ Expression parsing and evaluation
- ✅ Statement execution
- ✅ Variables and assignment
- ✅ Control flow
- ✅ Block scoping
- ✅ Functions
- ✅ Resolving and Binding
- 🔄 Classes (in progress)
- 🔄 Inheritance

## Running Examples

The repository includes several example Lox programs in the `examples/` directory:

```
examples/
├── assign.lox         - Basic variable declaration and assignment
├── branching.lox      - If/else control flow
├── fibonacci_for.lox  - Fibonacci sequence using for loops
├── fibonacci_while.lox - Fibonacci sequence using while loops
├── logical.lox        - Logical operators with short-circuit evaluation
├── print.lox          - Basic printing of different types
└── scope.lox          - Nested scope demonstration
```

To run an example:

```bash
cargo run -- examples/fibonacci_for.lox
```

Or directly with the binary:

```bash
./target/release/lox_interpreter_rs examples/fibonacci_for.lox
```

The GitHub Actions workflow automatically runs all examples as part of CI to verify interpreter functionality.

## What I Learned

This project served as a dual learning experience, deepening my understanding of both Rust and interpreter design:

### Rust Insights

- **Ownership System**: Implementing environments and variable scoping required careful consideration of Rust's ownership rules
- **Smart Pointers**: Using `Rc<RefCell<T>>` to create parent-pointer trees for lexical scoping
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
- **Error Recovery**: Implementing synchronization points for error recovery in the parser
- **Control Flow**: Implementing constructs like if/else, while, and for loops

## Acknowledgements

This project is based on Robert Nystrom's excellent book ["Crafting Interpreters"](https://craftinginterpreters.com/), which provides a clear and thorough explanation of interpreter implementation. The original Java code has been thoughtfully adapted to idiomatic Rust.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
