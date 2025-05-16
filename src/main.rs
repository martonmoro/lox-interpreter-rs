mod class;
mod environment;
mod error;
mod function;
mod interpreter;
mod object;
mod parser;
mod resolver;
mod scanner;
mod syntax;
mod token;

use std::env;
use std::fs::File;
use std::io::{self, BufRead, Read};
use std::process::exit;

use error::Error;
use interpreter::Interpreter;
use parser::Parser;
use resolver::Resolver;
use scanner::Scanner;

struct Lox {
    interpreter: Interpreter,
}

impl Lox {
    fn new() -> Self {
        Lox {
            interpreter: Interpreter::new(),
        }
    }

    fn run_file(&mut self, file_path: &String) -> Result<(), Error> {
        let mut file = File::open(file_path)?;
        let mut contents = String::new();

        file.read_to_string(&mut contents)?;

        self.run(contents)
    }

    fn run_prompt(&mut self) -> Result<(), Error> {
        let stdin = io::stdin();

        let handle = stdin.lock();

        for line in handle.lines() {
            self.run(line?)?;
            print!("> ");
        }

        Ok(())
    }

    fn run(&mut self, source: String) -> Result<(), Error> {
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens();

        let mut parser = Parser::new(tokens);
        let mut statements = parser.parse()?;

        // We don’t run the resolver if there are any parse errors. If the code
        // has a syntax error, it’s never going to run, so there’s little value
        // in resolving it. If the syntax is clean, we tell the resolver to do
        // its thing. The resolver has a reference to the interpreter and pokes
        // the resolution data directly into it as it walks over variables. When
        // the interpreter runs next, it has everything it needs.
        let mut resolver = Resolver::new(&mut self.interpreter);
        resolver.resolve_stmts(&statements);

        if resolver.had_error {
            return Ok(());
        }

        // We could go farther and report warnings for code that isn’t
        // necessarily wrong but probably isn’t useful. For example, many IDEs
        // will warn if you have unreachable code after a return statement, or a
        // local variable whose value is never read. All of that would be pretty
        // easy to add to our static visiting pass, or as separate passes.

        self.interpreter.interpret(&mut statements)?;

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let args: Vec<String> = env::args().collect();
    let mut lox = Lox::new();
    match &args[..] {
        [_, file_path] => match lox.run_file(file_path) {
            Ok(_) => (),
            Err(Error::Runtime { .. }) => exit(70),
            Err(Error::Return { .. }) => unreachable!(),
            Err(Error::Parse) => exit(65),
            Err(Error::Io(_)) => unimplemented!(),
        },
        [_] => lox.run_prompt()?,
        _ => {
            eprintln!("Usage: lox-rs [script]");
            exit(64)
        }
    }
    Ok(())
}
