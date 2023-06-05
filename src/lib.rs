//! Library for manipulations on context-free grammars. Most transformations are abstracted over
//! grammar representations.

#![recursion_limit="512"]

#![deny(
        missing_copy_implementations,
        trivial_casts,
        trivial_numeric_casts,
        unused_import_braces,
        unused_qualifications)]

#![cfg_attr(test, deny(warnings))]

#![cfg_attr(test, allow(missing_docs))]

#![cfg_attr(feature = "nightly",
            feature(
                collections,
                collections_range,
                core,
                nonzero,
            ))]

extern crate bit_matrix;
extern crate bit_vec;
extern crate optional;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[cfg(feature = "rand")]
extern crate rand;
#[cfg(feature = "num")]
extern crate num;

#[cfg(feature = "nightly")]
extern crate collections;
#[cfg(feature = "nightly")]
extern crate core;

mod analysis;
pub mod binarized;
pub mod classification;
pub mod earley;
pub mod generate;
mod grammar;
pub mod history;
pub mod precedence;
pub mod prediction;
pub mod remap;
pub mod rule;
pub mod sequence;
pub mod symbol;

pub use binarized::BinarizedCfg;
pub use grammar::{Cfg, ContextFree, ContextFreeRef, ContextFreeMut};
pub use rule::GrammarRule;
pub use symbol::Symbol;
