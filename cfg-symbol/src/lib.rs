//! A type that can represent symbols in a context-free grammar. Symbols are distinguished by their
//! IDs.

pub mod intern;
mod source;
mod symbol;

pub use self::source::SymbolName;
pub use self::source::SymbolSource;
pub use self::symbol::Symbol;
pub use self::symbol::SymbolPrimitive;
