use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

fn main() {
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("keywords.rs");
    let mut file = BufWriter::new(File::create(&path).unwrap());

    write!(
        &mut file,
        "pub static KEYWORDS: phf::Map<&'static str, TokenType> = {}",
        phf_codegen::Map::new()
            .entry("and", "TokenType::And")
            .entry("class", "TokenType::Class")
            .entry("else", "TokenType::Else")
            .entry("false", "TokenType::False")
            .entry("for", "TokenType::For")
            .entry("fun", "TokenType::Fun")
            .entry("if", "TokenType::If")
            .entry("nil", "TokenType::Nil")
            .entry("or", "TokenType::Or")
            .entry("print", "TokenType::Print")
            .entry("return", "TokenType::Return")
            .entry("super", "TokenType::Super")
            .entry("this", "TokenType::This")
            .entry("true", "TokenType::True")
            .entry("var", "TokenType::Var")
            .entry("while", "TokenType::While")
            .build()
    )
    .unwrap();
    writeln!(&mut file, ";").unwrap(); // Fixed as suggested by clippy
}
