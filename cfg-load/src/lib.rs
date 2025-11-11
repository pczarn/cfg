//! Allows us to load context-free grammars from
//! a BNF string.

#![deny(unsafe_code)]
#![deny(missing_docs)]

pub mod advanced;
pub mod basic;
mod string_interner;
// pub mod advanced_macro;

use std::fmt;

pub use crate::advanced::CfgLoadAdvancedExt;
pub use crate::basic::CfgLoadExt;

/// Represents an error when loading a BNF string.
#[derive(Debug, Clone)]
pub struct LoadError {
    /// Human-readable reason for the error.
    pub reason: String,
    /// Line where the error happened.
    ///
    /// One-indexed.
    pub line: u32,
    /// Column where the error happened.
    ///
    /// One-indexed.
    pub col: u32,
    /// Optionally, the token at which the error happened.
    pub token: Option<crate::advanced::Token>,
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LoadError {
                reason,
                line,
                col,
                token,
            } => {
                write!(
                    f,
                    "Parse error at line {} column {}: reason: {} token: {:?}",
                    line, col, reason, token
                )
            }
        }
    }
}

impl std::error::Error for LoadError {}
