//! Library for manipulations on context-free grammars. Most transformations are abstracted over
//! grammar representations.

#![deny(missing_docs,
        missing_copy_implementations,
        trivial_casts,
        trivial_numeric_casts,
        unused_import_braces,
        unused_qualifications)]
#![cfg_attr(test, deny(warnings))]
#![feature(
    collections, collections_range,
    core, nonzero,
    unboxed_closures
)]

extern crate bit_matrix;
extern crate bit_vec;
extern crate collections;
extern crate core;

mod binarized;
pub mod cycles;
mod grammar;
pub mod history;
pub mod precedence;
pub mod prediction;
mod rhs_closure;
mod rule;
pub mod rule_builder;
pub mod rule_container;
pub mod sequence;
pub mod sequence_builder;
mod sequence_destination;
pub mod symbol;
pub mod usefulness;

pub use binarized::BinarizedCfg;
pub use grammar::{Cfg, ContextFree, ContextFreeRef, ContextFreeMut};
pub use rule::GrammarRule;
pub use symbol::SymbolSource;
