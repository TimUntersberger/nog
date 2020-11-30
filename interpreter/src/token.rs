use logos::Logos;
use std::ops::Range;

#[derive(Logos, Debug, PartialEq, Clone, Eq, Hash)]
pub enum TokenKind {
    #[regex("[a-z$_][a-zA-Z0-9$_]*", priority = 2)]
    Identifier,
    #[regex("[A-Z][a-zA-Z0-9$_]*")]
    ClassIdentifier,
    #[regex("[0-9]+")]
    NumberLiteral,
    #[regex(r#""([^"\\]|\\t|\\u|\\n|\\")*""#)]
    StringLiteral,
    #[token("#")]
    Hash,
    #[token("++")]
    PlusPlus,
    #[token("+=")]
    PlusEqual,
    #[token("-=")]
    MinusEqual,
    #[token("*=")]
    StarEqual,
    #[token("/=")]
    SlashEqual,
    #[token("--")]
    MinusMinus,
    #[token("///")]
    TripleSlash,
    #[token("export")]
    Export,
    #[token("static")]
    Static,
    #[token("while")]
    While,
    #[token("var")]
    Var,
    #[token("=>")]
    Arrow,
    #[token("class")]
    Class,
    #[token("import")]
    Import,
    #[token("break")]
    Break,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("continue")]
    Continue,
    #[token("op")]
    Op,
    #[token("null")]
    Null,
    #[token("//")]
    Comment,
    #[token("fn")]
    Fn,
    #[regex("(true|false)")]
    BooleanLiteral,
    #[token("=")]
    Equal,
    #[token("return")]
    Return,
    #[token("elif")]
    ElseIf,
    #[token("if")]
    If,
    #[token("else", priority = 3)]
    Else,
    #[token(",")]
    Comma,
    #[token("!")]
    ExclamationMark,
    #[token(".")]
    Dot,
    #[token(":")]
    Colon,
    #[token("::")]
    DoubleColon,
    #[token("&&")]
    And,
    #[token("||")]
    Or,
    #[token(";")]
    SemiColon,
    #[token(">")]
    GT,
    #[token(">=")]
    GTE,
    #[token("<")]
    LT,
    #[token("<=")]
    LTE,
    #[token("==")]
    EQ,
    #[token("!=")]
    NEQ,
    #[token("(")]
    LParan,
    #[token(")")]
    RParan,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token("{")]
    LCurly,
    #[token("}")]
    RCurly,
    #[regex("\r?\n")]
    NewLine,
    #[regex(r"[ \t\f]+", logos::skip)]
    #[error]
    Error,
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct Token(pub TokenKind, pub Range<usize>);

impl Default for Token {
    fn default() -> Self {
        Token(TokenKind::Error, 0..0)
    }
}

impl Default for &Token {
    fn default() -> Self {
        &Token(TokenKind::Error, 0..0)
    }
}

impl From<(TokenKind, Range<usize>)> for Token {
    fn from(value: (TokenKind, Range<usize>)) -> Self {
        Self(value.0, value.1)
    }
}

#[cfg(test)]
mod test {
    use super::Token;
    use super::TokenKind::*;
    use crate::lexer::Lexer;

    fn parse<T: Into<Token>>(code: &str, expected: T) {
        let mut lexer = Lexer::new(code, 0);
        let actual = lexer.next();

        assert_eq!(actual, Some(expected.into()));
    }

    fn parse_seq<T: Into<Token>>(code: &str, expected: Vec<T>) {
        let mut lexer = Lexer::new(code, 0);

        for expected in expected {
            let actual = lexer.next();
            assert_eq!(actual, Some(expected.into()));
        }

        assert!(lexer.next().is_none());
    }

    #[test]
    fn else_kw() {
        parse("else", (Else, 0..4))
    }

    #[test]
    fn if_stmt_with_else() {
        parse_seq(
            "if true {} else {}",
            vec![
                (If, 0..2),
                (BooleanLiteral, 3..7),
                (LCurly, 8..9),
                (RCurly, 9..10),
                (Else, 11..15),
                (LCurly, 16..17),
                (RCurly, 17..18),
            ],
        )
    }

    #[test]
    fn if_stmt_with_else_if() {
        parse_seq(
            "if true {} elif true {}",
            vec![
                (If, 0..2),
                (BooleanLiteral, 3..7),
                (LCurly, 8..9),
                (RCurly, 9..10),
                (ElseIf, 11..15),
                (BooleanLiteral, 16..20),
                (LCurly, 21..22),
                (RCurly, 22..23),
            ],
        )
    }

    #[test]
    fn identifier() {
        parse("identifier", (Identifier, 0..10))
    }

    #[test]
    fn class_identifier() {
        parse("Identifier", (ClassIdentifier, 0..10))
    }
}
