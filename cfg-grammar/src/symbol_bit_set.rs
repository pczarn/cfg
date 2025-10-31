//! Informs whether symbols are terminal or nonterminal.

use std::{iter, ops};

use bit_vec;
use bit_vec::BitVec;

use crate::local_prelude::*;

/// A set of symbols in the form of a bit vector.
///
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(miniserde::Serialize, miniserde::Deserialize, Clone, Debug)]
pub struct SymbolBitSet {
    bit_vec: BitVec,
}

impl Default for SymbolBitSet {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolBitSet {
    /// Constructs an empty `SymbolBitSet`.
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

    /// Resets the set to `false` repeated as many times as there are symbols
    /// in the given source.
    ///
    /// In other words, none of the symbols are included, but we have space to mark them
    /// as included.
    pub fn reset(&mut self, symbol_source: &SymbolSource) {
        self.bit_vec = BitVec::new();
        self.bit_vec
            .extend(iter::repeat(false).take(symbol_source.num_syms()));
    }

    /// Resets the set to `true` repeated as many times as there are symbols
    /// in the given source.
    ///
    /// In other words, all of the symbols are included in the set.
    pub fn set_all(&mut self, symbol_source: &SymbolSource) {
        self.bit_vec = BitVec::new();
        self.bit_vec
            .extend(iter::repeat(true).take(symbol_source.num_syms()));
    }

    /// Makes this set include only symbols which appear anywhere in the grammar.
    pub fn used(&mut self, grammar: &Cfg) {
        self.reset(grammar.sym_source());
        for rule in grammar.rules() {
            self.set(rule.lhs, true);
            for &sym in &rule.rhs[..] {
                self.set(sym, true);
            }
        }
    }

    /// Makes this set include only symbols which do **not** appear
    /// anywhere in the grammar.
    pub fn unused(&mut self, grammar: &Cfg) {
        self.set_all(grammar.sym_source());
        for rule in grammar.rules() {
            self.set(rule.lhs, false);
            for &sym in &rule.rhs[..] {
                self.set(sym, false);
            }
        }
    }

    /// Included symbols will be excluded, and symbols that do not
    /// appear in the set will be included instead.
    pub fn negate(&mut self) {
        self.bit_vec.negate();
    }

    /// Gathers information about whether symbols are terminal or nonterminal.
    /// Constructs a set of terminal symbols.
    ///
    /// Constructs a data structure in O(n) time.
    pub fn terminal(&mut self, grammar: &Cfg) {
        self.set_all(grammar.sym_source());
        for rule in grammar.rules() {
            self.set(rule.lhs, false);
        }
    }

    /// Gathers information about whether symbols are nulling.
    /// Includes the set of nulling symbols.
    ///
    /// Constructs a data structure in O(n) time.
    pub fn nulling(&mut self, grammar: &Cfg) {
        if self.space() == 0 {
            self.reset(grammar.sym_source());
        }
        for rule in grammar.rules() {
            if rule.rhs.is_empty() {
                self.set(rule.lhs, true);
            }
        }
    }

    /// Gathers information about whether symbols are terminal or nonterminal.
    /// Includes the set of nonterminal symbols.
    ///
    /// Constructs a data structure in O(n) time.
    pub fn productive(&mut self, grammar: &Cfg) {
        if self.space() == 0 {
            self.reset(grammar.sym_source());
        }
        for rule in grammar.rules() {
            self.set(rule.lhs, true);
        }
    }

    /// Excludes all nonterminals from the given grammar
    /// from the set.
    pub fn subtract_productive(&mut self, grammar: &Cfg) {
        if self.space() == 0 {
            self.set_all(grammar.sym_source());
        }
        for rule in grammar.rules() {
            self.set(rule.lhs, false);
        }
    }

    /// Sets the membership for the given symbol.
    pub fn set(&mut self, index: Symbol, elem: bool) {
        self.bit_vec.set(index.usize(), elem);
    }

    /// Allows access to the underlying `BitVec`.
    pub fn bit_vec(&self) -> &BitVec {
        &self.bit_vec
    }

    /// Converts into a bit vector.
    pub fn into_bit_vec(self) -> BitVec {
        self.bit_vec
    }

    /// Iterates over symbols in the set.
    pub fn iter(&self) -> impl Iterator<Item = Symbol> + use<'_> {
        self.bit_vec
            .iter()
            .zip(SymbolSource::generate_fresh())
            .filter_map(|(is_present, sym)| if is_present { Some(sym) } else { None })
    }

    /// Includes all symbols that are included in the other set.
    pub fn union(&mut self, other: &SymbolBitSet) {
        self.bit_vec.or(&other.bit_vec);
    }

    /// Returns the number of symbols we have space for.
    pub fn space(&self) -> usize {
        self.bit_vec.len()
    }

    /// Determines whether this set contains all symbols we have
    /// space for.
    pub fn all(&self) -> bool {
        self.bit_vec.iter().all(|b| b)
    }

    /// Not to be confused with `fn is_empty`.
    pub fn is_empty(&self) -> bool {
        self.bit_vec.iter().any(|b| b)
    }

    /// Reserves space for additional symbols.
    pub fn reserve(&mut self, len: usize) {
        self.bit_vec
            .extend(iter::repeat(false).take(len.saturating_sub(self.bit_vec.len())));
    }
}

static TRUE: bool = true;
static FALSE: bool = false;

impl ops::Index<Symbol> for SymbolBitSet {
    type Output = bool;

    fn index(&self, index: Symbol) -> &Self::Output {
        if self.bit_vec[index.usize()] {
            &TRUE
        } else {
            &FALSE
        }
    }
}

impl Cfg {
    /// Constructs a set of terminal symbols.
    pub fn terminal_symbols(&self) -> SymbolBitSet {
        let mut set = SymbolBitSet::new();
        set.terminal(self);
        set
    }

    /// Constructs a set of nulling symbols.
    pub fn nulling_symbols(&self) -> SymbolBitSet {
        let mut set = SymbolBitSet::new();
        set.reset(self.sym_source());
        set.nulling(self);
        set
    }

    /// Constructs a set of unused symbols.
    pub fn unused_symbols(&self) -> SymbolBitSet {
        let mut set = SymbolBitSet::new();
        set.reset(self.sym_source());
        set.unused(self);
        set
    }
}
