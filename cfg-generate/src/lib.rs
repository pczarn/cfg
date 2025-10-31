//! Generate strings from a grammar.

#![deny(unsafe_code)]
#![deny(missing_docs)]

mod genetic;
#[cfg(feature = "weighted")]
pub mod weighted;
