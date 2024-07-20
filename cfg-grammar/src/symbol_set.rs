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

impl SymbolBitSet {
    /// Constructs a `SymbolBitSet`.
    pub fn new() -> Self {
        SymbolBitSet {
            bit_vec: BitVec::new(),
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
