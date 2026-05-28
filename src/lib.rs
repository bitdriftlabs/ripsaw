#![deny(warnings)]
#![warn(clippy::all)]
#![warn(clippy::arithmetic_side_effects)]

#[cfg(feature = "compiler")]
pub mod compiler;

#[cfg(feature = "compiler")]
pub use compiler::prelude;

#[cfg(feature = "value")]
pub mod value;

#[cfg(feature = "diagnostic")]
pub mod diagnostic;

#[cfg(feature = "path")]
pub mod path;

#[cfg(feature = "parser")]
pub mod parser;

#[cfg(feature = "core")]
pub mod core;

#[cfg(feature = "stdlib-base")]
pub mod stdlib;

#[cfg(feature = "stdlib-base")]
pub mod protobuf;

#[cfg(feature = "docs")]
pub mod docs;

#[cfg(feature = "cli")]
pub mod cli;

#[cfg(feature = "test_framework")]
pub mod test;

#[cfg(feature = "parsing")]
pub mod parsing;
