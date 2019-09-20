use core::nonzero::NonZero;

pub type SymbolRepr = u32;
/// The first usable symbol ID.
const FIRST_ID: SymbolRepr = 1;

/// A common grammar symbol type.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub struct Symbol(NonZero<SymbolRepr>);

impl From<SymbolRepr> for Symbol {
    #[inline]
    fn from(id: SymbolRepr) -> Self {
        let id = id + 1;
        unsafe { Symbol(NonZero::new(id)) }
    }
}

impl Into<SymbolRepr> for Symbol {
    #[inline]
    fn into(self) -> SymbolRepr {
        *self.0 as SymbolRepr - 1
    }
}
