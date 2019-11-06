//! Various lexer utilities

use rowan::{TextUnit};
use std::{
    fmt,
    iter,
    ops::Sub,
};

mod consumed;

pub use self::consumed::Consumed;

/// An error that can occur when lexing
#[derive(Debug, Clone, Copy)]
pub enum Error {
    /// An invalid token was detected
    UnexpectedInput,
    /// End of file was reached mid-token
    UnexpectedEOF,
}

/// Defines what a token is, a simple enum token kind complete with
/// the length it actually took in the source string.
pub trait Token {
    /// Return the length this token ended up taking in the source
    /// string. May vary even on the same token kind.
    fn len(&self) -> TextUnit;
}
impl<T> Token for (T, TextUnit) {
    fn len(&self) -> TextUnit {
        self.1
    }
}

/// Defines how to attach text length to a token type. This is first
/// used by `wrap` to attach the length of the text consumed, and then
/// the higher-level API `into_iter` converts the lengths to
/// subslices.
///
/// You shouldn't need to implement this yourself in most cases,
/// there's a default implementation for `Result<T, Error>` where `T`
/// implements `From<Error>`, that results in a simple `(T, TextUnit)`
/// tuple.
pub trait Attach {
    /// The output type
    type Output;

    /// Attach the specified text length to this token. The actual
    /// slices can be derived from the lengths later as this tree is
    /// lossless.
    fn attach(self, len: TextUnit) -> Self::Output;
}
impl<T: From<Error>> Attach for Result<T, Error> {
    type Output = (T, TextUnit);

    fn attach(self, len: TextUnit) -> Self::Output {
        match self {
            Ok(success) => (success, len),
            Err(err) => (err.into(), len),
        }
    }
}

/// A lexer state defines where in the input the lexer is. Basically a
/// fancy wrapper around `&str` that adds a few convenience functions
/// that slices away chunks of the string.
#[derive(Default, Copy, Clone)]
pub struct State<'a> {
    input: &'a str,
}
impl<'a> fmt::Debug for State<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.remaining())
    }
}
impl<'a> From<&'a str> for State<'a> {
    fn from(s: &'a str) -> Self {
        Self::new(s)
    }
}
impl<'a> State<'a> {
    /// Construct a new state ready to eat away `input`
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
        }
    }

    /// Return the remaining string to be lexed
    pub fn remaining(&self) -> &str {
        &self.input
    }
    /// Return the first character of the remaining string to be
    /// lexed, or an unexpected eof error.
    pub fn peek(self) -> Result<char, Error> {
        self.remaining().chars().next().ok_or(Error::UnexpectedEOF)
    }

    /// Consume a certain amount of bytes. This will rightfully panic
    /// if you use a offset that breaks between code points, or if
    /// it's outside the string.
    pub fn consume(&mut self, len: TextUnit) {
        self.input = &self.input[len.to_usize()..];
    }
    /// Consume the first character, panicking if called at the end of
    /// input (so always peek first)
    pub fn bump(&mut self) {
        let c = self.peek().expect("bumped past eof");
        self.consume(TextUnit::of_char(c));
    }
    /// Try consuming a string, if remaining input starts with
    /// it. Returns true if it ended up eating this string, or false
    /// if it didn't match.
    pub fn take(&mut self, s: &str) -> Consumed {
        if self.remaining().starts_with(s) {
            self.consume(TextUnit::of_str(s));
            s.chars().collect()
        } else {
            Consumed::zero()
        }
    }
    /// Consume as many characters as possible that meet a certain
    /// predicate, returning the amount of characters consumed.
    pub fn take_while<F>(&mut self, predicate: F) -> Consumed
    where
        F: Fn(char) -> bool,
    {
        let consumed: Consumed = self
            .remaining()
            .chars()
            .take_while(|&c| predicate(c))
            .collect();
        self.consume(consumed.bytes());
        consumed
    }
}
impl<'a> Sub for State<'a> {
    type Output = TextUnit;

    /// Inspect how far a state has eaten. This will compare the two
    /// states and how much further ahead this state is than the
    /// previous.
    ///
    /// ### Panics
    ///
    /// This function panics if the states don't both point to the
    /// same string, or if the previous state was actually further
    /// ahead than the current.
    fn sub(self, prev: Self) -> Self::Output {
        let self_ptr = self.input.as_ptr() as usize;
        let prev_ptr = self.input.as_ptr() as usize;
        assert!(
            self_ptr >= prev_ptr && self_ptr < prev_ptr + prev.remaining().len(),
            "these states must originate from the same source and self must be further ahead than previous"
        );
        TextUnit::of_str(prev.remaining()) - TextUnit::of_str(self.remaining())
    }
}

/// Wrap your core lexer function with some boilerplate.
///
/// 1. Return `None` if the string is empty.
/// 2. Save the current state
/// 3. Call your function.
/// 4. Obtain the string length that was lexed.
/// 5. Panic if you consumed nothing even though there was input left
///    to be lexed.
/// 6. Attach the string length to the return value using the `Attach`
///    trait.
pub fn wrap<'a, F, R, S>(s: S, f: F) -> Option<R::Output>
where
    F: FnOnce(&mut State) -> R,
    R: Attach,
    S: Into<State<'a>>,
{
    let start: State<'a> = s.into();
    if start.remaining().is_empty() {
        return None;
    }
    let mut state = start;
    let output = f(&mut state);
    let len = state - start;
    debug_assert!(
        len != TextUnit::from(0),
        "Lexer should not return an empty token, this would likely lead to an infinite loop"
    );
    Some(output.attach(len))
}

/// Lex a function repeatedly until it returns `None`. This may not
/// fit the needs of everyone, so be prepared to re-implement it if
/// needed. Luckily, it's simple.
///
/// 1. Take a token from the remaining string.
/// 2. Exit if the previous step returned `None`.
/// 3. Advance string with the taken length.
/// 4. Repeat everything since step 1.
pub fn into_iter<'a, F, T>(input: &'a str, mut f: F) -> impl Iterator<Item = T> + 'a
where
    T: Token,
    F: (FnMut(&str) -> Option<T>) + 'a,
{
    let mut remaining = input;
    iter::from_fn(move || {
        let token = f(remaining)?;
        let len = token.len().to_usize();
        remaining = &remaining[len..];
        Some(token)
    })
}

/// Wraps an iterator such as one produced by `into_iter` to returns
/// an iterator of string references. This is unfortunately
/// specialized for the type `(T, &str)` because it can't seem to be
/// made generic without having associated types have generic
/// lifetimes. Might need manual re-implementation.
pub fn string_slices<'a, I, T>(input: &'a str, iter: I) -> impl Iterator<Item = (T, &str)> + 'a
where
    I: IntoIterator<Item = (T, TextUnit)> + 'a,
{
    let iter = iter.into_iter();
    let mut remaining = input;
    iter.map(move |(token, len)| {
        let len = len.to_usize();
        let slice = &remaining[..len];
        remaining = &remaining[len..];
        (token, slice)
    })
}
