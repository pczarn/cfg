pub mod basic;
pub mod advanced;

use std::fmt;

pub use crate::basic::CfgLoadExt;
pub use crate::advanced::CfgLoadAdvancedExt;

#[derive(Debug, Clone)]
pub enum LoadError {
    Parse {
        reason: String,
        line: u32,
        col: u32,
        token: Option<crate::advanced::Token>,
    },
    Eval {
        reason: String,
    },
    Lex {
        reason: String,
    }
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LoadError::Parse { reason, line, col, token } => {
                write!(f, "Parse error at line {} column {}: reason: {} token: {:?}", line, col, reason, token)
            }
            LoadError::Eval { reason } => {
                write!(f, "Eval error. Reason: {}", reason)
            }
            LoadError::Lex { reason } => {
                write!(f, "Lexical grammar error. Reason: {}", reason)
            }
        }
    }
}
