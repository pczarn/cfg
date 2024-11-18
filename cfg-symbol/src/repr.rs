use core::num::{NonZeroU32, NonZeroU16, NonZeroU8};

pub type SymbolRepr = u32;
/// The first usable symbol ID.
pub const FIRST_ID: SymbolRepr = 0;
/// The first usable symbol ID.
pub const NULL_ID: SymbolRepr = u32::MAX;
pub const NULL_ID_16: u16 = u16::MAX;
pub const NULL_ID_8: u8 = u8::MAX;

pub trait Symbolic: Copy + Clone + ::std::fmt::Debug + Eq + Ord + PartialEq + PartialOrd + From<usize> + Into<usize> {
    /// Cast the symbol's ID to `usize`.
    fn usize(self) -> usize {
        self.into()
    }
}

/// A common grammar symbol type.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
// miniserde impls are further below
#[derive(Clone, Copy, Debug, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub struct Symbol {
    n: NonZeroU32,
}

/// A smaller symbol type.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
// miniserde impls are further below
#[derive(Clone, Copy, Debug, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub struct Symbol16 {
    n: NonZeroU16,
}

/// A smaller symbol type.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
// miniserde impls are further below
#[derive(Clone, Copy, Debug, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub struct Symbol8 {
    n: NonZeroU8,
}

impl Default for Symbol {
    fn default() -> Self {
        FIRST_ID.into()
    }
}

impl Default for Symbol16 {
    fn default() -> Self {
        (FIRST_ID as u16).into()
    }
}

impl Default for Symbol8 {
    fn default() -> Self {
        (FIRST_ID as u8).into()
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

impl From<u16> for Symbol16 {
    #[inline]
    fn from(id: u16) -> Self {
        debug_assert_ne!(
            id, NULL_ID_16,
            "invalid coversion from a null id to non-null Symbol"
        );
        Symbol16 {
            n: NonZeroU16::new(id.wrapping_add(1)).unwrap(),
        }
    }
}

impl From<u8> for Symbol8 {
    #[inline]
    fn from(id: u8) -> Self {
        debug_assert_ne!(
            id, NULL_ID_8,
            "invalid coversion from a null id to non-null Symbol"
        );
        Symbol8 {
            n: NonZeroU8::new(id.wrapping_add(1)).unwrap(),
        }
    }
}

impl From<Symbol> for SymbolRepr {
    #[inline]
    fn from(val: Symbol) -> SymbolRepr {
        val.n.get().wrapping_sub(1)
    }
}

impl From<Symbol16> for u16 {
    #[inline]
    fn from(val: Symbol16) -> u16 {
        val.n.get().wrapping_sub(1)
    }
}

impl From<Symbol8> for u8 {
    #[inline]
    fn from(val: Symbol8) -> u8 {
        val.n.get().wrapping_sub(1)
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

impl From<usize> for Symbol16 {
    #[inline]
    fn from(id: usize) -> Self {
        Symbol16::from(id as u16)
    }
}

impl From<Symbol16> for usize {
    #[inline]
    fn from(val: Symbol16) -> usize {
        let id: u16 = val.into();
        id as usize
    }
}

impl From<usize> for Symbol8 {
    #[inline]
    fn from(id: usize) -> Self {
        Symbol8::from(id as u8)
    }
}

impl From<Symbol8> for usize {
    #[inline]
    fn from(val: Symbol8) -> usize {
        let id: u8 = val.into();
        id as usize
    }
}

impl Symbolic for Symbol {}
impl Symbolic for Symbol16 {}
impl Symbolic for Symbol8 {}

#[cfg(feature = "miniserde")]
mod miniserde_impls {
    use super::{Symbol, Symbol16, Symbol8, SymbolRepr};
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

    impl Visitor for Place<Symbol16> {
        fn nonnegative(&mut self, n: u64) -> Result<()> {
            if n < u16::MAX as u64 {
                self.out = Some((n as u16).into());
                Ok(())
            } else {
                Err(Error)
            }
        }
    }

    impl Deserialize for Symbol16 {
        fn begin(out: &mut Option<Self>) -> &mut dyn de::Visitor {
            Place::new(out)
        }
    }

    impl Serialize for Symbol16 {
        fn begin(&self) -> ser::Fragment {
            let n: u16 = (*self).into();
            ser::Fragment::U64(n as u64)
        }
    }

    impl Visitor for Place<Symbol8> {
        fn nonnegative(&mut self, n: u64) -> Result<()> {
            if n < u8::MAX as u64 {
                self.out = Some((n as u8).into());
                Ok(())
            } else {
                Err(Error)
            }
        }
    }

    impl Deserialize for Symbol8 {
        fn begin(out: &mut Option<Self>) -> &mut dyn de::Visitor {
            Place::new(out)
        }
    }

    impl Serialize for Symbol8 {
        fn begin(&self) -> ser::Fragment {
            let n: u8 = (*self).into();
            ser::Fragment::U64(n as u64)
        }
    }
}
