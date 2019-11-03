//! Various lexer utilities

use rowan::{SmolStr, TextUnit};
use std::{
    fmt,
    ops::{Deref, DerefMut},
    ptr,
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

/// Defines how to attach text to a token type.
///
/// You shouldn't need to implement this yourself in most cases,
/// there's a default implementation for `Result<T, Error>` where `T`
/// implements `From<Error>`, that results in a simple `(T, SmolStr)`
/// tuple.
pub trait Attach {
    /// The output type
    type Output;

    /// Attach the specified text to this token
    fn attach(self, text: SmolStr) -> Self::Output;
}
impl<T: From<Error>> Attach for Result<T, Error> {
    type Output = (T, SmolStr);

    fn attach(self, text: SmolStr) -> Self::Output {
        match self {
            Ok(success) => (success, text),
            Err(err) => (err.into(), text),
        }
    }
}

/// A lexer state defines where in the input the lexer is. Basically a
/// fancy wrapper around `&str` that adds a few convenience functions
/// that slices away chunks of the string.
// Internally this actually keeps track of the whole string (as
// opposed to *literally* slicing away), in order to easily support
// comparing with a previous state.
#[derive(Default, Copy, Clone)]
pub struct State<'a> {
    input: &'a str,
    cursor: TextUnit,
}
impl<'a> fmt::Debug for State<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.remaining())
    }
}
impl<'a> State<'a> {
    /// Construct a new state ready to eat away `input`
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            cursor: TextUnit::from(0),
        }
    }

    /// Return the remaining string to be lexed
    pub fn remaining(&self) -> &str {
        &self.input[self.cursor.to_usize()..]
    }
    /// Return the first character of the remaining string to be
    /// lexed, or an unexpected eof error.
    pub fn peek(self) -> Result<char, Error> {
        self.remaining().chars().next().ok_or(Error::UnexpectedEOF)
    }
    /// Inspect the string between the current state and a previous
    /// state. Can be used to see how far a state has eaten.
    ///
    /// ### Panics
    ///
    /// This function panics if the states don't both point to the
    /// same string, or if the previous state was actually further
    /// ahead than the current.
    pub fn string_since(&'_ self, previous: &'_ State<'_>) -> &'_ str {
        assert!(
            ptr::eq(self.input, previous.input),
            "these states originate from the same source"
        );
        assert!(
            previous.cursor <= self.cursor,
            "previous state was further ahead than current state"
        );
        &self.input[previous.cursor.to_usize()..self.cursor.to_usize()]
    }

    /// Consume the first character, panicking if called at the end of
    /// input (so always peek first)
    pub fn bump(&mut self) {
        let c = self.peek().expect("bumped past eof");
        self.cursor += TextUnit::of_char(c);
    }
    /// Try consuming a string, if remaining input starts with
    /// it. Returns true if it ended up eating this string, or false
    /// if it didn't match.
    pub fn take(&mut self, s: &str) -> Consumed {
        if self.remaining().starts_with(s) {
            self.cursor += TextUnit::of_str(s);
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
        self.cursor += consumed.bytes();
        consumed
    }
}

/// A useful base lexer that can keep track of your state and wrap
/// your lexing functions.
#[derive(Debug)]
pub struct Base<'a> {
    state: State<'a>,
}
impl<'a> Base<'a> {
    /// Create a new instance, ready to wrap your functions with open
    /// arms.
    pub fn new(input: &'a str) -> Self {
        Self {
            state: State::new(input),
        }
    }

    /// Wrap your core lexer function with some boilerplate.
    ///
    /// 1. Return `None` if the string is empty.
    /// 2. Save the current state
    /// 3. Call your function.
    /// 4. Obtain the string that was lexed.
    /// 5. Panic if you returned an empty string even though there was
    ///    input left to be lexed.
    /// 6. Attach the string to the return value using the `Attach`
    ///    trait.
    pub fn wrap<F, R>(&mut self, f: F) -> Option<R::Output>
    where
        R: Attach,
        F: FnOnce(&mut Self) -> R,
    {
        let start = self.state;
        if start.remaining().is_empty() {
            return None;
        }
        let output = f(self);
        let since = self.state.string_since(&start);
        debug_assert!(
            !since.is_empty() || start.remaining().is_empty(),
            "Lexer should not return an empty token, this would likely lead to an infinite loop"
        );
        Some(output.attach(SmolStr::new(since)))
    }
}
impl<'a> Deref for Base<'a> {
    type Target = State<'a>;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}
impl<'a> DerefMut for Base<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}
