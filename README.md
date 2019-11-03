# rowan-tools

Abstracting away some boilerplate needed to get started creating
parsers for the awesome
[rowan](https://github.com/rust-analyzer/rowan) library.

## Background

### What this is

This is basically a series of useful structures and traits that can be
reused when writing parsers. Don't get too hyped, it's barely anything
at all. And that's the beauty of it, if you're anything like me and
want to write your parsers by hand to get rid of all magic, this won't
stop you. All this will do is make your hand-written code cleaner by
not including all boilerplate.

See examples in the `examples/` directory.

### Why this exists

I've been thinking too long and too hard over how to best get rid of
repetetive code [like this (from
rnix)](https://github.com/nix-community/rnix-parser/blob/master/src/tokenizer.rs#L70-L104)
when creating lexers and parsers in rowan. I've been thinking about
having a lexer generator procedural macro, incorporating
[nom](https://github.com/Geal/nom). I've even gone so far as to
imagining creating a programming language that can dynamically modify
its own syntax (like [Forth seems to
support](https://github.com/philburk/pforth/blob/c1a87b8298475c3fdd007b14a1413d2a6fd0fa61/fth/system.fth#L12-L14)).

In the end, I basically tricked myself into thinking the problem was
way bigger than it is. So I started doing that painful thing called
thinking again, from scratch. And the result is obvious: Getting
repetetive code away should be done by placing it in a
library. Simple, right? It's crazy how far your mind can wonder off if
you let it.

## Usage

### Writing a lexer

First, we need a definition for a token, basically an integer label
for what kind of word a piece of text is.

```rust
#[derive(Debug, PartialEq, Eq, Hash)]
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
```

Notice how we've made a token for `Error` and `Whitespace` as
well. This is one of the philosophies of rowan: Parsing should be
lossless (even for errnous trees!), with all kinds of comments and
whitespace still in the tree. This also leads to a nice property: All
span information (start/end position of each token) can be computed
later instead of eagerly attached to the tree. This also makes it
possible to modify the tree and re-compute all span information.

Secondly, we need to tell rowan-tools about our `Error` kind. This is
because one of the abstractions rowan-tools does is allow us to use
the awesome Rust facilities `Result` to fail when an invalid token is
detected.

```rust
impl From<Error> for TokenKind {
    fn from(_error: Error) -> Self {
        Self::Error
    }
}
```

Here we don't really care about what kind of error it was, but you can
make more token kinds if you want. Now we can get started with the
main lexer.

The lexer will input a string and output an iterator over tokens.
That's fairly broad! So, writing a lexer can be done in at least two
flavors:

- Making a struct that implements `Iterator`.
- Making a function using `std::iter::from_fn`.

For this example I'll choose the second approach as it's easiest for
something trivial.

```rust
fn lex(input: &'_ str) -> impl Iterator<Item = (TokenKind, SmolStr)> + '_ {
    iter::from_fn(move || {
        unimplemented!("TODO: Take a token")
    })
}
```

To minimize the amount of magic rowan-tools bring us, let's start
implementing a lexer from scratch and then compare the result.

```rust
let first = match input.chars().next() {
    Some(c) => c,
    None => return None,
};
match first {
    c if c.is_whitespace() => {
        let ws = input.chars()
            .take_while(|c| c.is_whitespace())
            .map(char::len_utf8)
            .sum();
        let string = SmolStr::new(&input[..ws]);
        input = &input[ws..];
        Some((TokenKind::Whitespace, string))
    },
    c if c == '.' || c.is_digit(10) => {
        let mut consume = input.chars()
            .take_while(|c| c.is_digit(10))
            .map(char::len_utf8)
            .sum();
        if input[consume..].starts_with(".") {
            consume += 1; // length of "."
            let trailing: usize = input[consume..].chars()
                .take_while(|c| c.is_digit(10))
                .map(char::len_utf8)
                .sum();
```

uuhh. Phew. That *is* boring. You know what, let me finish that as its
own file, `examples/lexer_fn_manual.rs`.

My point is, rowan-tools is not magic, like you might consider
nom[^1], pest[^2], or lalrpop[^3]. These parser applications are
great, but they the polar opposite of hand-writing your stuff.

Now, let's anyway consider rowan-tools. The parser will be the exact
same as the previous one I never completed.

```rust
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
            '+' => { base.bump(); Ok(TokenKind::Add) },
            _ => {
                base.bump();
                Err(Error::UnexpectedInput)
            },
        })
    })
}
```

Some notable differences are the `take*` functions, the
`.at_least(1)?` call which uses the `?` shorthand thanks to Rust's
`Result` types, and the lack of specifying which string we're
returning.

Thing is though, `iter::from_fn` requires our closure to return
`Option<(TokenKind, SmolStr)>` to return the value we want. So how can
we return an `Err(Error::UnexpectedInput)` and still get the same
result? The answer lies in `base.wrap(...)`, which wraps our function
with some code that

1. Checks for EOF for us.
1. Records what string was moved past during the function.
1. Uses the `Attach` trait to attach the string to the value.

Now we already have a fully functioning lexer. Let's test it out!

```
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
```

That was easy, wasn't it? And notice how absolutely no whitespace
information was lost, and an errnous number (`4.` syntax is not
allowed here) did not stop the entire lexer.

[^1]: https://github.com/Geal/nom
[^2]: https://github.com/pest-parser/pest
[^3]: https://github.com/lalrpop/lalrpop
