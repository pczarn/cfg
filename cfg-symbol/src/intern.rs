//! Interns symbols and implements symbol mappings.

use crate::*;

#[cfg(feature = "serialize")]
use miniserde::{Deserialize, Serialize};

/// Populates maps with new symbols.
pub struct Intern {
    pub source: SymbolSource,
    pub mapping: Mapping,
}

/// Contains maps for translation between internal and external symbols.
#[derive(Clone, Default, Debug)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct Mapping {
    /// An array of internal symbols, indexed by external symbol ID.
    pub to_internal: Vec<Option<Symbol>>,
    /// An array of external symbols, indexed by internal symbol ID.
    pub to_external: Vec<Symbol>,
}

impl Intern {
    pub fn new(num_external: usize) -> Self {
        Intern {
            source: SymbolSource::new(),
            mapping: Mapping::new(num_external),
        }
    }

    pub fn intern(&mut self, symbol: Symbol) -> Symbol {
        if let Some(internal) = self.mapping.to_internal[symbol.usize()] {
            internal
        } else {
            let [new_sym] = self.source.sym();
            self.mapping.to_internal[symbol.usize()] = Some(new_sym);
            assert_eq!(self.mapping.to_external.len(), new_sym.usize());
            self.mapping.to_external.push(symbol);
            new_sym
        }
    }
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
                other.to_internal[sym.usize()]
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
