//! This example does not rely on rowan-tools, and is used in the
//! README as an example for why you'd want to use rowan-tools.

use rowan_tools::rowan::SmolStr;
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

fn lex(mut input: &'_ str) -> impl Iterator<Item = (TokenKind, SmolStr)> + '_ {
    iter::from_fn(move || {
        let first = match input.chars().next() {
            Some(c) => c,
            None => return None,
        };
        match first {
            c if c.is_whitespace() => {
                let ws = input
                    .chars()
                    .take_while(|c| c.is_whitespace())
                    .map(char::len_utf8)
                    .sum();
                let string = SmolStr::new(&input[..ws]);
                input = &input[ws..];
                Some((TokenKind::Whitespace, string))
            },
            c if c == '.' || c.is_digit(10) => {
                let mut consume = input
                    .chars()
                    .take_while(|c| c.is_digit(10))
                    .map(char::len_utf8)
                    .sum();
                if input[consume..].starts_with(".") {
                    consume += 1; // length of "."
                    let trailing: usize = input[consume..]
                        .chars()
                        .take_while(|c| c.is_digit(10))
                        .map(char::len_utf8)
                        .sum();
                    consume += trailing;
                    let string = SmolStr::new(&input[..consume]);
                    input = &input[consume..];
                    if trailing == 0 {
                        Some((TokenKind::Error, string))
                    } else {
                        Some((TokenKind::Float, string))
                    }
                } else {
                    let string = SmolStr::new(&input[..consume]);
                    input = &input[consume..];
                    Some((TokenKind::Integer, string))
                }
            },
            '+' => {
                let string = SmolStr::new(&input[..1]);
                input = &input[1..];
                Some((TokenKind::Add, string))
            },
            c => {
                let len = c.len_utf8();
                let string = SmolStr::new(&input[..len]);
                input = &input[len..];
                Some((TokenKind::Error, string))
            },
        }
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
