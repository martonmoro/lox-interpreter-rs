mod error;
mod interpreter;
mod object;
mod parser;
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
use scanner::Scanner;

struct Lox {
    interpreter: Interpreter,
}

impl Lox {
    fn new() -> Self {
        Lox {
            interpreter: Interpreter {},
        }
    }

    fn run_file(&self, file_path: &String) -> Result<(), Error> {
        let mut file = File::open(file_path)?;
        let mut contents = String::new();

        file.read_to_string(&mut contents)?;

        self.run(contents)
    }

    fn run_prompt(&self) -> Result<(), Error> {
        let stdin = io::stdin();

        let handle = stdin.lock();

        for line in handle.lines() {
            self.run(line?)?;
            print!("> ");
        }

        Ok(())
    }

    fn run(&self, source: String) -> Result<(), Error> {
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens();

        let mut parser = Parser::new(tokens);
        let statements = parser.parse()?;
        self.interpreter.interpret(&statements)?;

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let args: Vec<String> = env::args().collect();
    let lox = Lox::new();
    match &args[..] {
        [_, file_path] => match lox.run_file(file_path) {
            Ok(_) => (),
            Err(Error::Runtime { .. }) => exit(70),
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
