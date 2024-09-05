use core::num::NonZeroU32;

#[cfg(feature = "serialize")]
use miniserde::de::{Deserialize, Visitor};
#[cfg(feature = "serialize")]
use miniserde::Serialize;
#[cfg(feature = "serialize")]
use miniserde::{make_place, Error, Result};

#[cfg(feature = "serialize")]
make_place!(Place);

pub type SymbolRepr = u32;
/// The first usable symbol ID.
pub const FIRST_ID: SymbolRepr = 0;
/// The first usable symbol ID.
pub const NULL_ID: SymbolRepr = !0;

/// A common grammar symbol type.
#[derive(Clone, Copy, Debug, Hash, Eq, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct Symbol(NonZeroU32);

impl Default for Symbol {
    fn default() -> Self {
        FIRST_ID.into()
    }
}

impl From<SymbolRepr> for Symbol {
    #[inline]
    fn from(id: SymbolRepr) -> Self {
        debug_assert_ne!(
            id, NULL_ID,
            "invalid coversion from a null id to non-null Symbol"
        );
        Symbol(NonZeroU32::new(id.wrapping_add(1)).unwrap())
    }
}

impl Into<SymbolRepr> for Symbol {
    #[inline]
    fn into(self) -> SymbolRepr {
        self.0.get().wrapping_sub(1)
    }
}

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

#[cfg(feature = "serialize")]
impl Visitor for Place<Symbol> {
    fn nonnegative(&mut self, n: u64) -> Result<()> {
        if n < ::std::u32::MAX as u64 {
            self.out = Some((n as SymbolRepr).into());
            Ok(())
        } else {
            Err(Error)
        }
    }
}

#[cfg(feature = "serialize")]
impl Deserialize for Symbol {
    fn begin(out: &mut Option<Self>) -> &mut dyn miniserde::de::Visitor {
        Place::new(out)
    }
}

#[cfg(feature = "serialize")]
impl Serialize for Symbol {
    fn begin(&self) -> miniserde::ser::Fragment {
        let n: u32 = (*self).into();
        miniserde::ser::Fragment::U64(n as u64)
    }
}
