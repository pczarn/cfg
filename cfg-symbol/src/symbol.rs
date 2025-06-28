//! TODO: Allow choice between u8, u16 and u32 for symbols
//! 

use std::num::{NonZeroU16, NonZeroU32, NonZeroU8};

pub trait SymbolPrimitive: TryFrom<NonZeroU32> + Into<NonZeroU32> + Copy {
    type BasePrimitive: From<Self> + TryInto<Self> + From<u8>;
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

/// A common grammar symbol type.
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
    pub fn first() -> Self {
        let one: T::BasePrimitive = 1u8.into();
        Symbol {
            n: one.try_into().ok().expect("unreachable: could not convert 1u8"),
        }
    }

    pub fn usize(&self) -> usize {
        self.n.into().get() as usize - 1
    }
}

impl<T: SymbolPrimitive> Into<u32> for Symbol<T> {
    fn into(self) -> u32 {
        let nzu32: NonZeroU32 = self.n.into();
        nzu32.get() - 1
    }
}

#[cfg(feature = "miniserde")]
mod miniserde_impls {
    use super::{Symbol, SymbolPrimitive};
    use std::num::NonZeroU32;
    use miniserde::de::{Deserialize, Visitor};
    use miniserde::{de, ser, Serialize};
    use miniserde::{make_place, Error, Result};

    make_place!(Place);

    impl<T: SymbolPrimitive> Visitor for Place<Symbol<T>> {
        fn nonnegative(&mut self, n: u64) -> Result<()> {
            if n < T::MAX as u64 {
                if let Some(Ok(nonzero_num)) = NonZeroU32::new(n as u32).map(|n| TryInto::<T>::try_into(n)) {
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
        fn begin(&self) -> ser::Fragment {
            let n: u32 = (*self).into();
            ser::Fragment::U64(n as u64)
        }
    }
}
