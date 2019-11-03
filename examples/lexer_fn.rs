use rowan_tools::{
    lexer::{Base, Error},
    rowan::SmolStr,
};
use std::iter;

#[derive(Debug, PartialEq, Eq)]
enum TokenKind {
    // Meta
    Error,
    Whitespace,

    // Operators
    Add,

    // Types
    Float,
    Integer,
}

impl From<Error> for TokenKind {
    fn from(_error: Error) -> Self {
        Self::Error
    }
}

fn lex(input: &'_ str) -> impl Iterator<Item = (TokenKind, SmolStr)> + '_ {
    let mut base = Base::new(input);

    iter::from_fn(move || {
        base.wrap(|base| match base.peek().unwrap() {
            c if c.is_whitespace() => {
                base.take_while(char::is_whitespace);
                Ok(TokenKind::Whitespace)
            },
            c if c == '.' || c.is_digit(10) => {
                base.take_while(|c| c.is_digit(10));
                if base.take(".").any() {
                    base.take_while(|c| c.is_digit(10)).at_least(1)?;
                    Ok(TokenKind::Float)
                } else {
                    Ok(TokenKind::Integer)
                }
            },
            '+' => {
                base.bump();
                Ok(TokenKind::Add)
            },
            _ => {
                base.bump();
                Err(Error::UnexpectedInput)
            },
        })
    })
}

#[rustfmt::skip]
fn main() {
    let mut lexer = lex("1 + 2.3 + 4. + .5");
    assert_eq!(lexer.next(), Some((TokenKind::Integer,    SmolStr::new("1"))));
    assert_eq!(lexer.next(), Some((TokenKind::Whitespace, SmolStr::new(" "))));
    assert_eq!(lexer.next(), Some((TokenKind::Add,        SmolStr::new("+"))));
    assert_eq!(lexer.next(), Some((TokenKind::Whitespace, SmolStr::new(" "))));
    assert_eq!(lexer.next(), Some((TokenKind::Float,      SmolStr::new("2.3"))));
    assert_eq!(lexer.next(), Some((TokenKind::Whitespace, SmolStr::new(" "))));
    assert_eq!(lexer.next(), Some((TokenKind::Add,        SmolStr::new("+"))));
    assert_eq!(lexer.next(), Some((TokenKind::Whitespace, SmolStr::new(" "))));
    assert_eq!(lexer.next(), Some((TokenKind::Error,      SmolStr::new("4."))));
    assert_eq!(lexer.next(), Some((TokenKind::Whitespace, SmolStr::new(" "))));
    assert_eq!(lexer.next(), Some((TokenKind::Add,        SmolStr::new("+"))));
    assert_eq!(lexer.next(), Some((TokenKind::Whitespace, SmolStr::new(" "))));
    assert_eq!(lexer.next(), Some((TokenKind::Float,      SmolStr::new(".5"))));
    assert_eq!(lexer.next(), None);
}
