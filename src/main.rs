mod error;
mod scanner;
mod token;

use std::env;
use std::fs::File;
use std::io::{self, BufRead, Read};
use std::process::exit;

use scanner::Scanner;

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let args: Vec<String> = env::args().collect();

    match &args[..] {
        [_, file_path] => run_file(file_path)?,
        [_] => run_prompt()?,
        _ => {
            eprintln!("Usage: jlox [script]");
            exit(64);
        }
    }
    Ok(())
}

fn run_file(file_path: &String) -> io::Result<()> {
    let mut file = File::open(file_path)?;
    let mut contents = String::new();

    file.read_to_string(&mut contents)?;

    run(contents)
}

fn run_prompt() -> io::Result<()> {
    let stdin = io::stdin();

    let handle = stdin.lock();

    for line in handle.lines() {
        run(line?)?;
        print!("> ");
    }

    Ok(())
}

fn run(source: String) -> io::Result<()> {
    let mut scanner = Scanner::new(source);

    let tokens = scanner.scan_tokens();

    for token in tokens {
        println!("{}", token);
    }

    Ok(())
}
