//! This example does not rely on rowan-tools, and is used in the
//! README as an example for why you'd want to use rowan-tools.

use rowan_tools::rowan::TextUnit;
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

fn lex(remaining: &'_ str) -> Option<(TokenKind, TextUnit)> {
    let first = match remaining.chars().next() {
        Some(c) => c,
        None => return None,
    };
    match first {
        c if c.is_whitespace() => {
            let ws: TextUnit = remaining
                .chars()
                .take_while(|c| c.is_whitespace())
                .map(TextUnit::of_char)
                .sum();
            Some((TokenKind::Whitespace, ws))
        },
        c if c == '.' || c.is_digit(10) => {
            let mut consume: TextUnit = remaining
                .chars()
                .take_while(|c| c.is_digit(10))
                .map(TextUnit::of_char)
                .sum();
            if remaining[consume.to_usize()..].starts_with(".") {
                consume += TextUnit::of_str(".");
                let trailing: TextUnit = remaining[consume.to_usize()..]
                    .chars()
                    .take_while(|c| c.is_digit(10))
                    .map(TextUnit::of_char)
                    .sum();
                consume += trailing;
                if trailing == TextUnit::from(0) {
                    Some((TokenKind::Error, consume))
                } else {
                    Some((TokenKind::Float, consume))
                }
            } else {
                Some((TokenKind::Integer, consume))
            }
        },
        '+' => {
            Some((TokenKind::Add, TextUnit::from(1)))
        },
        c => {
            Some((TokenKind::Error, TextUnit::of_char(c)))
        },
    }
}

fn tokenize_str(input: &'_ str) -> impl Iterator<Item = (TokenKind, &str)> + '_ {
    let mut remaining = input;
    iter::from_fn(move || {
        let (token, len) = lex(remaining)?;
        let rem = &remaining[..len.to_usize()];
        remaining = &remaining[len.to_usize()..];
        Some((token, rem))
    })
}

#[rustfmt::skip]
fn main() {
    let mut lexer = tokenize_str("1 + 2.3 + 4. + .5");
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
