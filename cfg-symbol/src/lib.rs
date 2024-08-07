//! A type that can represent symbols in a context-free grammar. Symbols are distinguished by their
//! IDs.

pub mod intern;
mod repr;
pub mod source;

pub use self::repr::Symbol;
pub use self::source::SymbolSource;
