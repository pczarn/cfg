//! Source

use std::{borrow::Cow, cell::Cell, iter};

use super::repr::{SymbolRepr, FIRST_ID, NULL_ID};

use crate::*;

thread_local! {
    static NEXT_ID_DEBUG: Cell<SymbolRepr> = Cell::new(FIRST_ID);
}

/// A source of numeric symbols.
#[allow(missing_copy_implementations)]
#[derive(Clone, Debug, Default)]
pub struct SymbolSource {
    next_id: SymbolRepr,
    names: Vec<Option<&'static str>>,
}

impl SymbolSource {
    /// Creates a source of numeric symbols with an empty symbol space.
    pub fn new() -> Self {
        Self { next_id: FIRST_ID, names: vec![] }
    }
    /// Returns generated symbols.
    pub fn sym<const N: usize>(&mut self) -> [Symbol; N] {
        let mut result = [Default::default(); N];
        for dest in &mut result {
            *dest = self.next_sym();
        }
        self.names.extend([None; N]);
        result
    }
    /// Returns generated symbols.
    pub fn sym_with_names<const N: usize>(&mut self, names: [&'static str; N]) -> [Symbol; N] {
        let mut result = [Default::default(); N];
        for dest in &mut result {
            *dest = self.next_sym();
        }
        self.names.extend(names.map(Some));
        result
    }
    /// Generates a new unique symbol.
    pub fn next_sym(&mut self) -> Symbol {
        let ret = self.next_id.into();
        self.next_id += 1;
        debug_assert_ne!(self.next_id, NULL_ID, "ran out of Symbol space?");
        ret
    }
    /// Generates a new unique symbol.
    pub fn next_sym_with_name(&mut self, name: &'static str) -> Symbol {
        let ret = self.next_id.into();
        self.next_id += 1;
        debug_assert_ne!(self.next_id, NULL_ID, "ran out of Symbol space?");
        self.names.push(Some(name));
        ret
    }
    pub fn name_of(&self, sym: Symbol) -> Cow<'static, str> {
        match self.names.get(sym.usize()).copied() {
            Some(Some(name)) => {
                Cow::Borrowed(name)
            }
            Some(None) => {
                Cow::Owned(format!("g{}", sym.usize()))
            }
            None => {
                panic!("unkown {:?}, we have only {} names", sym, self.names.len())
            }
        }
    }
    pub fn original_name_of(&self, sym: Symbol) -> Option<&'static str> {
        self.names.get(sym.usize()).copied().unwrap_or(None)
    }
    /// Returns the number of symbols in use.
    pub fn num_syms(&self) -> usize {
        self.next_id as usize
    }
    /// Returns an iterator that generates symbols.
    pub fn generate(&mut self) -> Generate {
        Generate { source: self }
    }

    // Translates symbol names to new symbol IDs.
    pub fn remap_symbols<F>(&mut self, mut map: F)
    where
        F: FnMut(Symbol) -> Symbol,
    {
        let mut new_names = vec![];
        for (sym, &name) in self.names.iter().enumerate() {
            let new_sym = map(sym.into()).usize();
            new_names.extend(iter::repeat(None).take((new_sym + 1).saturating_sub(new_names.len())));
            new_names[new_sym] = name;
        }
        self.names = new_names;
    }

    pub fn truncate(&mut self, new_len: usize) {
        self.names.truncate(new_len);
        self.next_id = new_len as u32;
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
