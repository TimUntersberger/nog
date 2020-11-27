use super::token::Token;
use logos::Logos;

#[derive(Clone)]
pub struct Lexer<'a> {
    inner: logos::Lexer<'a, Token<'a>>,
    prev: Option<Token<'a>>,
    current: Option<Token<'a>>,
}

impl<'a> std::fmt::Debug for Lexer<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(
            f,
            "Lexer {{\ncurrent: {:#?}\nprev: {:#?}\n}}",
            self.current, self.prev
        )
    }
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            inner: Token::lexer(source),
            current: None,
            prev: None,
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.prev = self.current.clone();
        self.current = self.inner.next();
        self.current.clone()
    }
}
