use std::{borrow::Cow, ops::Range};

use logos::{Lexer, Logos};

#[derive(Debug, PartialEq, Clone)]
pub struct TokenData<T = ()> {
    pub value: T,
    pub location: Range<usize>,
}

impl<T> TokenData<T> {
    pub fn new(value: T, location: Range<usize>) -> Self {
        Self { value, location }
    }
}

impl<T: Default> Default for TokenData<T> {
    fn default() -> Self {
        Self {
            value: Default::default(),
            location: 0..0,
        }
    }
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
    #[token("++", str_token_data)]
    PlusPlus(TokenData<&'a str>),
    #[token("--", str_token_data)]
    MinusMinus(TokenData<&'a str>),
    #[token("///", str_token_data)]
    TripleSlash(TokenData<&'a str>),
    #[token("export", str_token_data)]
    Export(TokenData<&'a str>),
    #[token("static", str_token_data)]
    Static(TokenData<&'a str>),
    #[token("while", str_token_data)]
    While(TokenData<&'a str>),
    #[token("var", str_token_data)]
    Var(TokenData<&'a str>),
    #[token("=>", str_token_data)]
    Arrow(TokenData<&'a str>),
    #[token("class", str_token_data)]
    Class(TokenData<&'a str>),
    #[token("import", str_token_data)]
    Import(TokenData<&'a str>),
    #[token("break", str_token_data)]
    Break(TokenData<&'a str>),
    #[token("+", str_token_data)]
    Plus(TokenData<&'a str>),
    #[token("-", str_token_data)]
    Minus(TokenData<&'a str>),
    #[token("*", str_token_data)]
    Star(TokenData<&'a str>),
    #[token("/", str_token_data)]
    Slash(TokenData<&'a str>),
    #[token("continue", str_token_data)]
    Continue(TokenData<&'a str>),
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
    #[token("=", str_token_data)]
    Equal(TokenData<&'a str>),
    #[token("return", str_token_data)]
    Return(TokenData<&'a str>),
    #[token("elif", str_token_data)]
    ElseIf(TokenData<&'a str>),
    #[token("if", str_token_data)]
    If(TokenData<&'a str>),
    #[token("else", str_token_data, priority = 3)]
    Else(TokenData<&'a str>),
    #[token(",", str_token_data)]
    Comma(TokenData<&'a str>),
    #[token(".", str_token_data)]
    Dot(TokenData<&'a str>),
    #[token(":", str_token_data)]
    Colon(TokenData<&'a str>),
    #[token("::", str_token_data)]
    DoubleColon(TokenData<&'a str>),
    #[token("&&", str_token_data)]
    And(TokenData<&'a str>),
    #[token("||", str_token_data)]
    Or(TokenData<&'a str>),
    #[token(";", str_token_data)]
    SemiColon(TokenData<&'a str>),
    #[token(">", str_token_data)]
    GT(TokenData<&'a str>),
    #[token(">=", str_token_data)]
    GTE(TokenData<&'a str>),
    #[token("<", str_token_data)]
    LT(TokenData<&'a str>),
    #[token("<=", str_token_data)]
    LTE(TokenData<&'a str>),
    #[token("==", str_token_data)]
    EQ(TokenData<&'a str>),
    #[token("!=", str_token_data)]
    NEQ(TokenData<&'a str>),
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
            | Token::LT(x)
            | Token::LTE(x)
            | Token::GT(x)
            | Token::GTE(x)
            | Token::EQ(x)
            | Token::NEQ(x)
            | Token::Colon(x)
            | Token::DoubleColon(x)
            | Token::TripleSlash(x)
            | Token::Export(x)
            | Token::Static(x)
            | Token::Hash(x)
            | Token::Return(x)
            | Token::Plus(x)
            | Token::Minus(x)
            | Token::Star(x)
            | Token::Slash(x)
            | Token::Comment(x)
            | Token::While(x)
            | Token::Import(x)
            | Token::Continue(x)
            | Token::SemiColon(x)
            | Token::LParan(x)
            | Token::RParan(x)
            | Token::LBracket(x)
            | Token::RBracket(x)
            | Token::LCurly(x)
            | Token::And(x)
            | Token::Or(x)
            | Token::PlusPlus(x)
            | Token::MinusMinus(x)
            | Token::RCurly(x)
            | Token::Break(x)
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

#[cfg(test)]
mod test {
    use super::Token::*;
    use super::{Token, TokenData};
    use logos::Logos;

    fn parse(code: &str, expected: Token) {
        let mut lexer = Token::lexer(code);
        let actual = lexer.next();

        assert_eq!(actual, Some(expected));
    }

    fn parse_seq(code: &str, expected: Vec<Token>) {
        let mut lexer = Token::lexer(code);

        for expected in expected {
            let actual = lexer.next();
            assert_eq!(actual, Some(expected));
        }

        assert!(lexer.next().is_none());
    }

    #[test]
    fn else_kw() {
        parse("else", Else(TokenData::new("else", 0..4)))
    }

    #[test]
    fn if_stmt_with_else() {
        parse_seq(
            "if true {} else {}",
            vec![
                If(TokenData::new("if", 0..2)),
                BooleanLiteral(TokenData::new(true, 3..7)),
                LCurly(TokenData::new("{", 8..9)),
                RCurly(TokenData::new("}", 9..10)),
                Else(TokenData::new("else", 11..15)),
                LCurly(TokenData::new("{", 16..17)),
                RCurly(TokenData::new("}", 17..18)),
            ],
        )
    }

    #[test]
    fn if_stmt_with_else_if() {
        parse_seq(
            "if true {} elif true {}",
            vec![
                If(TokenData::new("if", 0..2)),
                BooleanLiteral(TokenData::new(true, 3..7)),
                LCurly(TokenData::new("{", 8..9)),
                RCurly(TokenData::new("}", 9..10)),
                ElseIf(TokenData::new("elif", 11..15)),
                BooleanLiteral(TokenData::new(true, 16..20)),
                LCurly(TokenData::new("{", 21..22)),
                RCurly(TokenData::new("}", 22..23)),
            ],
        )
    }

    #[test]
    fn identifier() {
        parse(
            "identifier",
            Identifier(TokenData::new("identifier", 0..10)),
        )
    }

    #[test]
    fn class_identifier() {
        parse(
            "Identifier",
            ClassIdentifier(TokenData::new("Identifier", 0..10)),
        )
    }
}
