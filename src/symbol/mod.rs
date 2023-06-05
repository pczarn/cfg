//! A type that can represent symbols in a context-free grammar. Symbols are distinguished by their
//! IDs.

#[cfg_attr(feature = "nightly", path = "repr_nightly.rs")]
mod repr;
pub mod set;
pub mod source;

use std::convert::{From, Into};

pub use self::repr::Symbol;
use self::repr::SymbolRepr;
pub use self::set::SymbolBitSet;
pub use self::source::SymbolSource;

impl Symbol {
    /// Cast the symbol's ID to `usize`.
    #[inline]
    pub fn usize(self) -> usize {
        self.into()
    }
}

impl From<usize> for Symbol {
    #[inline]
    fn from(id: usize) -> Self {
        Symbol::from(id as SymbolRepr)
    }
}

impl Into<usize> for Symbol {
    #[inline]
    fn into(self) -> usize {
        let id: SymbolRepr = self.into();
        id as usize
    }
}
