//! Library for manipulations on context-free grammars. Most transformations are abstracted over
//! grammar representations.

#![deny(missing_docs,
        missing_copy_implementations,
        trivial_casts,
        trivial_numeric_casts,
        unused_import_braces,
        unused_qualifications)]

#![cfg_attr(test, deny(warnings))]

#![cfg_attr(feature = "nightly",
            feature(
                collections,
                collections_range,
                core,
                nonzero,
            ))]

extern crate bit_matrix;
extern crate bit_vec;

#[cfg(feature = "nightly")]
extern crate collections;
#[cfg(feature = "nightly")]
extern crate core;

mod binarized;
pub mod cycles;
mod grammar;
pub mod history;
pub mod precedence;
pub mod prediction;
mod rhs_closure;
pub mod rule;
pub mod sequence;
pub mod symbol;
pub mod usefulness;

pub use binarized::BinarizedCfg;
pub use grammar::{Cfg, ContextFree, ContextFreeRef, ContextFreeMut};
pub use rule::GrammarRule;
pub use symbol::SymbolSource;
