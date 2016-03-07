pub type SymbolRepr = u32;
/// The first usable symbol ID.
pub const FIRST_ID: SymbolRepr = 0;

/// A common grammar symbol type.
#[derive(Clone, Copy, Debug, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub struct Symbol(SymbolRepr);

impl From<SymbolRepr> for Symbol {
    #[inline]
    fn from(id: SymbolRepr) -> Self {
        Symbol(id)
    }
}

impl Into<SymbolRepr> for Symbol {
    #[inline]
    fn into(self) -> SymbolRepr {
        self.0
    }
}
