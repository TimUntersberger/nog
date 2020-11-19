use super::token::Token;
use logos::Logos;

pub struct Lexer<'a> {
    inner: logos::Lexer<'a, Token<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            inner: Token::lexer(source),
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}
