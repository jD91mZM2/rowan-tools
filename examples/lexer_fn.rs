use rowan_tools::{
    lexer::{self, Error},
    rowan::{TextUnit},
};

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

fn lex(remaining: &str) -> Option<(TokenKind, TextUnit)> {
    lexer::wrap(remaining, |state| match state.peek().unwrap() {
        c if c.is_whitespace() => {
            state.take_while(char::is_whitespace);
            Ok(TokenKind::Whitespace)
        },
        c if c == '.' || c.is_digit(10) => {
            state.take_while(|c| c.is_digit(10));
            if state.take(".").any() {
                state.take_while(|c| c.is_digit(10)).at_least(1)?;
                Ok(TokenKind::Float)
            } else {
                Ok(TokenKind::Integer)
            }
        },
        '+' => {
            state.bump();
            Ok(TokenKind::Add)
        },
        _ => {
            state.bump();
            Err(Error::UnexpectedInput)
        },
    })
}

fn tokenize(input: &'_ str) -> impl Iterator<Item = (TokenKind, &'_ str)> + '_ {
    lexer::string_slices(input, lexer::into_iter(input, lex))
}

#[rustfmt::skip]
fn main() {
    let mut lexer = tokenize("1 + 2.3 + 4. + .5");
    assert_eq!(lexer.next(), Some((TokenKind::Integer,    "1")));
    assert_eq!(lexer.next(), Some((TokenKind::Whitespace, " ")));
    assert_eq!(lexer.next(), Some((TokenKind::Add,        "+")));
    assert_eq!(lexer.next(), Some((TokenKind::Whitespace, " ")));
    assert_eq!(lexer.next(), Some((TokenKind::Float,      "2.3")));
    assert_eq!(lexer.next(), Some((TokenKind::Whitespace, " ")));
    assert_eq!(lexer.next(), Some((TokenKind::Add,        "+")));
    assert_eq!(lexer.next(), Some((TokenKind::Whitespace, " ")));
    assert_eq!(lexer.next(), Some((TokenKind::Error,      "4.")));
    assert_eq!(lexer.next(), Some((TokenKind::Whitespace, " ")));
    assert_eq!(lexer.next(), Some((TokenKind::Add,        "+")));
    assert_eq!(lexer.next(), Some((TokenKind::Whitespace, " ")));
    assert_eq!(lexer.next(), Some((TokenKind::Float,      ".5")));
    assert_eq!(lexer.next(), None);
}
