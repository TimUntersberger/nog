use super::token::{Token, TokenKind};
use logos::Logos;

#[derive(Clone)]
pub struct Lexer<'a> {
    inner: logos::Lexer<'a, TokenKind>,
    offset: usize,
    prev: Option<Token>,
    current: Option<Token>,
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
    pub fn new(source: &'a str, offset: usize) -> Self {
        Self {
            inner: TokenKind::lexer(source),
            offset,
            current: None,
            prev: None,
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.prev = self.current.clone();
        self.current = self
            .inner
            .next()
            .map(|kind| {
                let mut span = self.inner.span();
                span.start += self.offset;
                span.end += self.offset;

                (kind, span).into()
            });
        self.current.clone()
    }
}
