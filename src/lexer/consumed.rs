use super::Error;

use std::iter::{FromIterator, IntoIterator};

use rowan::TextUnit;

/// A structure containing the amount of characters and amount of
/// bytes that was consumed by a state operation.
#[derive(Debug, Clone, Copy)]
pub struct Consumed {
    chars: usize,
    bytes: TextUnit,
}
impl Consumed {
    /// A consumed instance that has consumed nothing
    pub fn zero() -> Self {
        Self {
            chars: 0,
            bytes: TextUnit::from(0),
        }
    }
    /// Returns the amount of characters consumed
    pub fn chars(self) -> usize {
        self.chars
    }
    /// Returns the amount of bytes consumed
    pub fn bytes(self) -> TextUnit {
        self.bytes
    }
    /// Returns true if a non-zero number of bytes were consumed
    pub fn any(self) -> bool {
        self.bytes > TextUnit::from(0)
    }
    /// Returns `Ok(self)` if the amount of characters consumed was at
    /// least `n`, otherwise `Err(Error::UnexpectedInput)`.
    pub fn at_least(self, n: usize) -> Result<Self, Error> {
        if self.chars >= n {
            Ok(self)
        } else {
            Err(Error::UnexpectedInput)
        }
    }
    /// Returns `Ok(self)` if the amount of characters consumed was at
    /// most `n`, otherwise `Err(Error::UnexpectedInput)`.
    pub fn at_most(self, n: usize) -> Result<Self, Error> {
        if self.chars <= n {
            Ok(self)
        } else {
            Err(Error::UnexpectedInput)
        }
    }
}
impl FromIterator<char> for Consumed {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = char>,
    {
        let mut consumed = Self {
            chars: 0,
            bytes: TextUnit::from(0),
        };
        for c in iter {
            consumed.chars += 1;
            consumed.bytes += TextUnit::of_char(c);
        }
        consumed
    }
}
