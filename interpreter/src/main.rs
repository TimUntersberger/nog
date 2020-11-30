use interpreter::Interpreter;
use parser::Parser;

#[macro_use]
#[allow(unused_macros)]
mod macros {
    /// Converts the given value into the inner value of the variant
    macro_rules! cast {
        ($enum: expr, $variant: path) => {
            if let $variant(x) = $enum {
                x
            } else {
                unreachable!()
            }
        };
    }

    /// Converts the given value into a number
    macro_rules! number {
        ($enum: expr) => {
            cast!($enum, Dynamic::Number)
        };
    }

    /// Converts the given value into an array
    macro_rules! array {
        ($enum: expr) => {
            cast!($enum, Dynamic::Array)
        };
    }

    macro_rules! hashmap {
        (@single $($x:tt)*) => (());
        (@count $($rest:expr),*) => (<[()]>::len(&[$(hashmap!(@single $rest)),*]));

        ($($key:expr => $value:expr,)+) => { hashmap!($($key => $value),+) };
        ($($key:expr => $value:expr),*) => {
            {
                let _cap = hashmap!(@count $($key),*);
                let mut _map = ::std::collections::HashMap::with_capacity(_cap);
                $(
                    let _ = _map.insert($key, $value);
                 )*
                    _map
            }
        };
    }
}

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
            // dbg!(&program);

            interpreter.execute(&program);
        }
        Err(e) => {
            println!("ERROR: {}", e);
        }
    };
}
