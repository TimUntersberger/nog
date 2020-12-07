use crate::interpreter::Interpreter;
use itertools::Itertools;
use parser::Parser;

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
mod scope;
mod token;

pub fn main() {
    let root_path = [
        std::env::current_dir().unwrap().to_str().unwrap(),
        "interpreter",
        "nog",
        "main.nog",
    ]
    .iter()
    .collect();

    let mut parser = Parser::new();
    let mut interpreter = Interpreter::new();

    let content = std::fs::read_to_string(&root_path).unwrap();

    parser.set_source(root_path, &content, 0);

    match parser.parse() {
        Ok(program) => {
            program.print();
            interpreter.execute(&program);
        }
        Err(e) => {
            println!("ERROR: {}", e);
        }
    };
}
