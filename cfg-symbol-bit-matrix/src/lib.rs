#![deny(unsafe_code)]

pub mod remap_symbols;
mod symbol_bit_matrix;

pub use self::remap_symbols::{CfgRemapSymbolsExt, Remap};
pub use self::symbol_bit_matrix::*;
