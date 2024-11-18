//! Library for manipulations on context-free grammars. Most transformations are abstracted over
//! grammar representations.

#![deny(unsafe_code)]
#![deny(
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications
)]
#![cfg_attr(test, deny(warnings))]
#![cfg_attr(test, allow(missing_docs))]

#[cfg(feature = "smallvec")]
use smallvec::SmallVec;

pub mod cfg;
mod occurence_map;
pub mod precedenced_rule;
pub mod rule_builder;
pub mod symbol_bit_set;

pub use crate::cfg::*;
pub use crate::symbol_bit_set::SymbolBitSet;

#[cfg(not(feature = "smallvec"))]
type MaybeSmallVec<T, const N: usize = 0> = Vec<T>;
#[cfg(feature = "smallvec")]
type MaybeSmallVec<T, const N: usize = 8> = SmallVec<[T; N]>;

mod local_prelude {
    pub use crate::precedenced_rule::PrecedencedRuleBuilder;
    pub use crate::*;
    pub use cfg_history::HistoryId;
    pub use cfg_symbol::source::SymbolSource;
    pub use cfg_symbol::{Symbol, Symbolic};
}
