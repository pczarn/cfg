//! This crate defines the type [`SymbolBitMatrix`] as well as
//! operations and utilites relating to it.

#![deny(unsafe_code)]
#![deny(missing_docs)]

pub mod remap_symbols;
mod symbol_bit_matrix;

pub use self::remap_symbols::{CfgRemapSymbolsExt, Remap};
pub use self::symbol_bit_matrix::*;
