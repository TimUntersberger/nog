use interpreter::Interpreter;
use parser::Parser;

#[macro_use]
#[allow(unused_macros)]
mod macros {
    macro_rules! consume {
        ($lexer: expr, $kind: path, $str: tt) => {
            if let Some(token) = $lexer.next() {
                if let $kind(_) = token {
                    Ok(token)
                } else {
                    Err(format!("Expected {} found {}", $str, token.text()))
                }
            } else {
                Err(format!("Expected {} found EOF", $str))
            }
        };
        ($lexer: expr, $kind: path, $str: tt, true) => {{
            while let Some(token) = $lexer.peek() {
                match token {
                    Token::NewLine(_) => $lexer.next(),
                    _ => break,
                };
            }
            consume!($lexer, $kind, $str)
        }};
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

/// # Ideas
///
/// ## Comment code
///
/// What if we have something like `///` in rust for nog. This could make it possible to extract
/// documentation from the AST, if we ever need to support generating documentation/lsp.
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

    parser.set_source(root_path, &content);

    let program = parser.parse();

        dbg!(&program);

    interpreter.execute(&program);
}
