//! Informs whether symbols are terminal or nonterminal.

use std::{iter, ops};

use bit_vec;
use bit_vec::BitVec;

use crate::local_prelude::*;

/// A set of symbols in the form of a bit vector.
pub struct SymbolBitSet {
    bit_vec: BitVec,
}

/// An iterator over a symbol set.
pub struct Iter<'a> {
    iter: iter::Enumerate<bit_vec::Iter<'a>>,
}

impl Default for SymbolBitSet {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolBitSet {
    /// Constructs a `SymbolBitSet`.
    pub fn new() -> Self {
        SymbolBitSet {
            bit_vec: BitVec::new(),
        }
    }
    /// Constructs a `SymbolBitSet`.
    pub fn from_elem(grammar: &Cfg, elem: bool) -> Self {
        SymbolBitSet {
            bit_vec: BitVec::from_elem(grammar.num_syms(), elem),
        }
    }

    fn initialize(&mut self, symbol_source: &SymbolSource) {
        todo!("{:?}", symbol_source)
    }

    pub fn used(&mut self, grammar: &Cfg) {
        for rule in grammar.rules() {
            self.set(rule.lhs, true);
            for &sym in rule.rhs {
                self.set(sym, true);
            }
        }
    }

    /// Gathers information about whether symbols are terminal or nonterminal.
    /// Constructs a set of terminal symbols.
    ///
    /// Constructs a data structure in O(n) time.
    pub fn terminal(&mut self, grammar: &Cfg) {
        self.initialize(grammar.sym_source());
        for rule in grammar.rules() {
            self.set(rule.lhs, false);
        }
    }

    /// Gathers information about whether symbols are terminal or nonterminal.
    /// Constructs a set of terminal symbols.
    ///
    /// Constructs a data structure in O(n) time.
    pub fn nulling(&mut self, grammar: &Cfg) {
        self.initialize(grammar.sym_source());
        for rule in grammar.rules() {
            if rule.rhs.is_empty() {
                self.set(rule.lhs, true);
            }
        }
    }

    /// Gathers information about whether symbols are terminal or nonterminal.
    /// Constructs a set of terminal symbols.
    ///
    /// Constructs a data structure in O(n) time.
    pub fn productive(&mut self, grammar: &Cfg) {
        self.initialize(grammar.sym_source());
        for rule in grammar.rules() {
            self.set(rule.lhs, true);
        }
    }

    pub fn set(&mut self, index: Symbol, elem: bool) {
        self.bit_vec.set(index.usize(), elem);
    }

    /// Converts into a bit vector.
    pub fn into_bit_vec(self) -> BitVec {
        self.bit_vec
    }

    /// Iterates over symbols in the set.
    pub fn iter(&self) -> Iter {
        Iter {
            iter: self.bit_vec.iter().enumerate(),
        }
    }

    pub fn union(&mut self, other: &SymbolBitSet) {
        self.bit_vec.or(&other.bit_vec);
    }

    pub fn len(&self) -> usize {
        self.bit_vec.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bit_vec.is_empty()
    }

    pub fn all(&self) -> bool {
        self.bit_vec.iter().all(|b| b)
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = Symbol;
    fn next(&mut self) -> Option<Self::Item> {
        for (id, is_present) in &mut self.iter {
            if is_present {
                return Some(Symbol::from(id));
            }
        }
        None
    }
}

static TRUE: bool = true;
static FALSE: bool = false;

impl ops::Index<Symbol> for SymbolBitSet {
    type Output = bool;

    fn index(&self, index: Symbol) -> &Self::Output {
        if self.bit_vec[index.into()] {
            &TRUE
        } else {
            &FALSE
        }
    }
}
