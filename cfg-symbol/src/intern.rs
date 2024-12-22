//! Interns symbols and implements symbol mappings.

use crate::*;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "miniserde")]
use miniserde::{Deserialize as MiniDeserialize, Serialize as MiniSerialize};

/// Contains maps for translation between internal and external symbols.
#[derive(Clone, Default, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "miniserde", derive(MiniSerialize, MiniDeserialize))]
pub struct Mapping {
    /// An array of internal symbols, indexed by external symbol ID.
    pub to_internal: Vec<Option<Symbol>>,
    /// An array of external symbols, indexed by internal symbol ID.
    pub to_external: Vec<Symbol>,
}

impl Mapping {
    /// Creates a new instance of `Mapping`.
    pub fn new(num_external: usize) -> Self {
        Mapping {
            to_internal: vec![None; num_external],
            to_external: vec![],
        }
    }

    /// Translates symbols in this map using another symbol map.
    /// This map becomes a combination of both mappings.
    pub fn translate(&mut self, other: &Self) {
        // For mapping to internal.
        for internal in &mut self.to_internal[..] {
            *internal = if let Some(sym) = *internal {
                other.to_internal.get(sym.usize()).and_then(|&x| x)
            } else {
                None
            };
        }
        // For mapping to external.
        let remapped = other
            .to_external
            .iter()
            .map(|middle| self.to_external[middle.usize()])
            .collect();
        self.to_external = remapped;
    }
}
