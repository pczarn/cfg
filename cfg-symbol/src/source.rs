//! Source

use super::repr::{SymbolRepr, FIRST_ID, NULL_ID};

use crate::*;

/// A source of numeric symbols.
#[allow(missing_copy_implementations)]
#[derive(Clone, Debug, Default)]
pub struct SymbolSource {
    next_id: SymbolRepr,
}

impl SymbolSource {
    /// Creates a source of numeric symbols with an empty symbol space.
    pub fn new() -> Self {
        Self { next_id: FIRST_ID }
    }
    /// Returns generated symbols.
    pub fn sym<const N: usize>(&mut self) -> [Symbol; N] {
        let mut result = [Default::default(); N];
        for dest in &mut result {
            *dest = self.next_sym();
        }
        result
    }
    /// Generates a new unique symbol.
    pub fn next_sym(&mut self) -> Symbol {
        let ret = self.next_id.into();
        self.next_id = self.next_id + 1;
        debug_assert_ne!(self.next_id, NULL_ID, "ran out of Symbol space?");
        ret
    }
    /// Returns the number of symbols in use.
    pub fn num_syms(&self) -> usize {
        self.next_id as usize
    }
    /// Returns an iterator that generates symbols.
    pub fn generate(&mut self) -> Generate {
        Generate { source: self }
    }
}

/// Iterator for generating symbols.
pub struct Generate<'a> {
    source: &'a mut SymbolSource,
}

impl<'a> Iterator for Generate<'a> {
    type Item = Symbol;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.source.next_sym())
    }
}
