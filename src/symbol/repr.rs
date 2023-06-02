pub type SymbolRepr = u32;
/// The first usable symbol ID.
pub const FIRST_ID: SymbolRepr = 0;

/// A common grammar symbol type.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, Hash, Eq, Ord, PartialEq, PartialOrd)]
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

#[test]
fn test_symbol_size() {
  assert_eq!(::std::mem::size_of::<Symbol>() * 8, 32);
}
