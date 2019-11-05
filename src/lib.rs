//! # rowan-tools
//!
//! Abstracting away some boilerplate needed to get started creating
//! parsers for the awesome
//! [rowan](https://github.com/rust-analyzer/rowan) library.

#![warn(
    // Harden built-in lints
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,

    // Harden clippy lints
    clippy::cargo_common_metadata,
    clippy::clone_on_ref_ptr,
    clippy::dbg_macro,
    clippy::decimal_literal_representation,
    clippy::float_cmp_const,
    clippy::get_unwrap,
    clippy::integer_arithmetic,
    clippy::integer_division,
    clippy::pedantic,
    clippy::print_stdout,
)]

pub use rowan;

pub mod lexer;
