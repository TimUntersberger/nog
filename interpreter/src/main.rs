use crate::interpreter::Interpreter;
use parser::Parser;
use std::path::PathBuf;

#[macro_use]
mod macros;

mod ast;
mod class;
mod dynamic;
mod expr_parser;
mod expression;
mod formatter;
mod function;
mod interpreter;
mod lexer;
mod method;
mod module;
mod operator;
mod parser;
mod runtime_error;
mod scope;
mod token;

pub fn main() {
    let root_dir: PathBuf = [
        std::env::current_dir().unwrap().to_str().unwrap(),
        "interpreter",
        "nog",
    ]
    .iter()
    .collect();

    let mut root_path = root_dir.clone();
    root_path.push("main.ns");

    let mut parser = Parser::new();
    let mut interpreter = Interpreter::new();

    interpreter.source_locations.push(root_dir);

    let content = std::fs::read_to_string(&root_path).unwrap();

    parser.set_source(root_path, &content, 0);

    match parser.parse() {
        Ok(program) => {
            program.print();
            if let Err(e) = interpreter.execute(&program) {
                println!("RUNTIME ERROR: {}", e.message());
            };
        }
        Err(e) => {
            println!("PARSE ERROR: {}", e);
        }
    };
}
