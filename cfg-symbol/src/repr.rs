use core::num::NonZeroU32;

pub type SymbolRepr = u32;
/// The first usable symbol ID.
pub const FIRST_ID: SymbolRepr = 0;
/// The first usable symbol ID.
pub const NULL_ID: SymbolRepr = u32::MAX;

/// A common grammar symbol type.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub struct Symbol {
    n: NonZeroU32,
}

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
        Symbol {
            n: NonZeroU32::new(id.wrapping_add(1)).unwrap(),
        }
    }
}

impl From<Symbol> for SymbolRepr {
    #[inline]
    fn from(val: Symbol) -> SymbolRepr {
        val.n.get().wrapping_sub(1)
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

impl From<Symbol> for usize {
    #[inline]
    fn from(val: Symbol) -> usize {
        let id: SymbolRepr = val.into();
        id as usize
    }
}

#[cfg(feature = "miniserde")]
mod miniserde_impls {
    use super::{Symbol, SymbolRepr};
    use miniserde::de::{Deserialize, Visitor};
    use miniserde::{de, ser, Serialize};
    use miniserde::{make_place, Error, Result};

    make_place!(Place);

    impl Visitor for Place<Symbol> {
        fn nonnegative(&mut self, n: u64) -> Result<()> {
            if n < SymbolRepr::MAX as u64 {
                self.out = Some((n as SymbolRepr).into());
                Ok(())
            } else {
                Err(Error)
            }
        }
    }

    impl Deserialize for Symbol {
        fn begin(out: &mut Option<Self>) -> &mut dyn de::Visitor {
            Place::new(out)
        }
    }

    impl Serialize for Symbol {
        fn begin(&self) -> ser::Fragment {
            let n: u32 = (*self).into();
            ser::Fragment::U64(n as u64)
        }
    }
}
