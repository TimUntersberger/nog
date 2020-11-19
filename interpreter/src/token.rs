use std::{borrow::Cow, ops::Range};

use logos::{Lexer, Logos};

#[derive(Debug, PartialEq, Clone)]
pub struct TokenData<T = ()> {
    pub value: T,
    pub location: Range<usize>,
}

fn str_token_data<'a>(lexer: &mut Lexer<'a, Token<'a>>) -> TokenData<&'a str> {
    TokenData {
        value: lexer.slice(),
        location: lexer.span(),
    }
}

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Token<'a> {
    #[regex("[a-z$_][a-zA-Z0-9$_]*", str_token_data, priority = 2)]
    Identifier(TokenData<&'a str>),
    #[regex("[A-Z][a-zA-Z0-9$_]*", str_token_data)]
    ClassIdentifier(TokenData<&'a str>),
    #[regex("[0-9]+", |lexer| {
        Some(TokenData {
            value: lexer.slice().parse::<i32>().ok()?,
            location: lexer.span()
        })
    })]
    NumberLiteral(TokenData<i32>),
    #[regex(r#""([^"\\]|\\t|\\u|\\n|\\")*""#, |lexer| {
        let location = lexer.span();

        Some(TokenData {
            value: &lexer.slice()[Range {
                start: 1,
                end: location.end - 1 - location.start
            }],
            location
        })
    })]
    StringLiteral(TokenData<&'a str>),
    #[token("#", str_token_data)]
    Hash(TokenData<&'a str>),
    #[token("export", str_token_data)]
    Export(TokenData<&'a str>),
    #[token("static", str_token_data)]
    Static(TokenData<&'a str>),
    #[token("var", str_token_data)]
    Var(TokenData<&'a str>),
    #[token("=>", str_token_data)]
    Arrow(TokenData<&'a str>),
    #[token("class", str_token_data)]
    Class(TokenData<&'a str>),
    #[token("import", str_token_data)]
    Import(TokenData<&'a str>),
    #[token("op", str_token_data)]
    Op(TokenData<&'a str>),
    #[token("null", str_token_data)]
    Null(TokenData<&'a str>),
    #[token("//", str_token_data)]
    Comment(TokenData<&'a str>),
    #[token("fn", str_token_data)]
    Fn(TokenData<&'a str>),
    #[regex("(true|false)", |lexer| Some(TokenData {
        value: lexer.slice() == "true",
        location: lexer.span()
    }))]
    BooleanLiteral(TokenData<bool>),
    #[regex("(\\+|-|!|\\?|\\*|/)", str_token_data)]
    Symbol(TokenData<&'a str>),
    #[token("=", str_token_data)]
    Equal(TokenData<&'a str>),
    #[token("return", str_token_data)]
    Return(TokenData<&'a str>),
    #[token("else if", str_token_data)]
    ElseIf(TokenData<&'a str>),
    #[token("if", str_token_data)]
    If(TokenData<&'a str>),
    #[token("else", str_token_data)]
    Else(TokenData<&'a str>),
    #[token(",", str_token_data)]
    Comma(TokenData<&'a str>),
    #[token(".", str_token_data)]
    Dot(TokenData<&'a str>),
    #[token(":", str_token_data)]
    Colon(TokenData<&'a str>),
    #[token("::", str_token_data)]
    DoubleColon(TokenData<&'a str>),
    #[token(";", str_token_data)]
    SemiColon(TokenData<&'a str>),
    #[token("(", str_token_data)]
    LParan(TokenData<&'a str>),
    #[token(")", str_token_data)]
    RParan(TokenData<&'a str>),
    #[token("[", str_token_data)]
    LBracket(TokenData<&'a str>),
    #[token("]", str_token_data)]
    RBracket(TokenData<&'a str>),
    #[token("{", str_token_data)]
    LCurly(TokenData<&'a str>),
    #[token("}", str_token_data)]
    RCurly(TokenData<&'a str>),
    #[regex("\r?\n", str_token_data)]
    NewLine(TokenData<&'a str>),
    #[regex(r"[ \t\f]+", logos::skip)]
    #[error]
    Error,
}

impl<'a> Token<'a> {
    pub fn is_whitespace(&self) -> bool {
        match self {
            Token::NewLine(_) => true,
            _ => false,
        }
    }
    pub fn text(&'a self) -> Cow<'a, str> {
        match self {
            Token::Identifier(x)
            | Token::ClassIdentifier(x)
            | Token::Colon(x)
            | Token::DoubleColon(x)
            | Token::Export(x)
            | Token::Static(x)
            | Token::Hash(x)
            | Token::Return(x)
            | Token::Comment(x)
            | Token::Import(x)
            | Token::Symbol(x)
            | Token::SemiColon(x)
            | Token::LParan(x)
            | Token::RParan(x)
            | Token::LBracket(x)
            | Token::RBracket(x)
            | Token::LCurly(x)
            | Token::RCurly(x)
            | Token::NewLine(x)
            | Token::Equal(x)
            | Token::Arrow(x)
            | Token::Comma(x)
            | Token::If(x)
            | Token::ElseIf(x)
            | Token::Else(x)
            | Token::Fn(x)
            | Token::Null(x)
            | Token::Class(x)
            | Token::Op(x)
            | Token::Var(x)
            | Token::Dot(x) => x.value.into(),
            Token::StringLiteral(x) => format!("\"{}\"", x.value).into(),
            Token::NumberLiteral(x) => x.value.to_string().into(),
            Token::BooleanLiteral(x) => x.value.to_string().into(),
            Token::Error => panic!("Don't know how to do error yet"),
        }
    }
}
