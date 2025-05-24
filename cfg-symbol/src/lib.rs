//! A type that can represent symbols in a context-free grammar. Symbols are distinguished by their
//! IDs.

pub mod intern;
mod symbol;
mod source;

pub use self::symbol::Symbol;
pub use self::symbol::SymbolPrimitive;
pub use self::source::SymbolSource;
