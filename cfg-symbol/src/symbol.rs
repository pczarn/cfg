//! Definitions for our grammar symbol type.
//!
//! A symbol can be though of as simply an integer,
//! which only works for the `SymbolSource` where it
//! was grabbed from, or a `SymbolSource` with a similar
//! symbol at that integer value, such as one in a cloned grammar.
//! In short, best to be careful not to mix symbols between different
//! grammars.
//!
//! # TODO
//!
//! Allow choice between u8, u16 and u32 for symbols.

use std::num::{NonZeroU8, NonZeroU16, NonZeroU32};

///
pub trait SymbolPrimitive: TryFrom<NonZeroU32> + Into<NonZeroU32> + Copy {
    /// Unsigned integer type that covers all possible values of `Self`.
    type BasePrimitive: From<Self> + TryInto<Self> + From<u8>;

    /// Highest available numeric value for `Self`.
    ///
    /// For example, for `NonZeroU16`, this will be `u16::MAX`.
    const MAX: u32;
}

impl SymbolPrimitive for NonZeroU8 {
    type BasePrimitive = u8;
    const MAX: u32 = u8::MAX as u32;
}

impl SymbolPrimitive for NonZeroU16 {
    type BasePrimitive = u16;
    const MAX: u32 = u16::MAX as u32;
}

impl SymbolPrimitive for NonZeroU32 {
    type BasePrimitive = u32;
    const MAX: u32 = u32::MAX;
}

/// Our common grammar symbol type.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
// miniserde impls are further below
#[derive(Clone, Copy, Debug, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub struct Symbol<T: SymbolPrimitive = NonZeroU32> {
    pub(crate) n: T,
}

impl<T: SymbolPrimitive> Default for Symbol<T> {
    fn default() -> Self {
        Self::first()
    }
}

impl<T: SymbolPrimitive> Symbol<T> {
    /// Returns the symbol with the lowest numeric value.
    pub fn first() -> Self {
        let one: T::BasePrimitive = 1u8.into();
        Symbol {
            n: one
                .try_into()
                .ok()
                .expect("unreachable: could not convert 1u8"),
        }
    }

    /// Returns the symbol's numeric value.
    ///
    /// # Panics
    ///
    /// May fail on 16-bit platforms in case of overflow for `usize`.
    pub fn usize(&self) -> usize {
        let val = self.n.into().get();
        #[cfg(target_pointer_width = "16")]
        assert!(val <= 0xFFFF);
        val as usize - 1
    }
}

impl Symbol {
    /// Constructs the `Symbol` from its numeric value.
    ///
    /// # Correctness
    ///
    /// Best to avoid using this function. Instead, grab the `Symbol`s
    /// from `SymbolSource`, possibly through `generate_fresh`.
    ///
    /// # Panics
    ///
    /// Panics if the numeric value is `u32::MAX`.
    pub fn from_raw(n: u32) -> Self {
        Symbol {
            n: (n + 1).try_into().unwrap(),
        }
    }
}

impl<T: SymbolPrimitive> Into<u32> for Symbol<T> {
    fn into(self) -> u32 {
        let nzu32: NonZeroU32 = self.n.into();
        nzu32.get() - 1
    }
}

mod miniserde_impls {
    use super::{Symbol, SymbolPrimitive};
    use miniserde::de::{Deserialize, Visitor};
    use miniserde::{Error, Result, make_place};
    use miniserde::{Serialize, de, ser};
    use std::num::NonZeroU32;

    make_place!(Place);

    impl<T: SymbolPrimitive> Visitor for Place<Symbol<T>> {
        fn nonnegative(&mut self, n: u64) -> Result<()> {
            if n < T::MAX as u64 {
                if let Some(Ok(nonzero_num)) =
                    NonZeroU32::new((n + 1) as u32).map(|n| TryInto::<T>::try_into(n))
                {
                    self.out = Some(Symbol { n: nonzero_num });
                    Ok(())
                } else {
                    Err(Error)
                }
            } else {
                Err(Error)
            }
        }
    }

    impl<T: SymbolPrimitive> Deserialize for Symbol<T> {
        fn begin(out: &mut Option<Self>) -> &mut dyn de::Visitor {
            Place::new(out)
        }
    }

    impl<T: SymbolPrimitive> Serialize for Symbol<T> {
        fn begin(&self) -> ser::Fragment<'_> {
            let n: u32 = (*self).into();
            ser::Fragment::U64(n as u64)
        }
    }
}

#[cfg(feature = "nanoserde")]
impl nanoserde::DeBin for Symbol<NonZeroU32> {
    fn de_bin(offset: &mut usize, bytes: &[u8]) -> Result<Self, nanoserde::DeBinErr> {
        Ok(Symbol {
            n: u32::de_bin(offset, bytes)?.try_into().unwrap(),
        })
    }
}

#[cfg(feature = "nanoserde")]
impl nanoserde::SerBin for Symbol<NonZeroU32> {
    fn ser_bin(&self, output: &mut Vec<u8>) {
        u32::ser_bin(&self.n.get(), output);
    }
}
