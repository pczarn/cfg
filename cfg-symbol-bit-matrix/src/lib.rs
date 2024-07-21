use std::ops::{self, Deref, DerefMut};

use bit_matrix::BitMatrix;

use cfg_grammar::Cfg;
use cfg_symbol::Symbol;

pub struct SymbolBitMatrix {
    bit_matrix: BitMatrix,
}

impl SymbolBitMatrix {
    pub fn new(num_syms: usize) -> Self {
        SymbolBitMatrix {
            bit_matrix: BitMatrix::new(num_syms, num_syms),
        }
    }

    pub fn set(&mut self, row: Symbol, col: Symbol, included: bool) {
        self.bit_matrix.set(row.usize(), col.usize(), included);
    }

    /// Returns the direct derivation matrix.
    pub fn direct_derivation_matrix(grammar: &Cfg) -> Self {
        let mut derivation = Self::new(grammar.sym_source().num_syms());

        for rule in grammar.rules() {
            derivation.set(rule.lhs, rule.lhs, true);
            for &sym in rule.rhs {
                derivation.set(rule.lhs, sym, true);
            }
        }
        derivation
    }

    /// Returns the derivation matrix.
    pub fn reachability_matrix(grammar: &Cfg) -> Self {
        let mut result = Self::direct_derivation_matrix(grammar);
        result.transitive_closure();
        result.reflexive_closure();
        result
    }

    pub fn iter_row_syms(&self, row: Symbol) -> impl Iterator<Item = Symbol> + '_ {
        self.bit_matrix
            .iter_row(row.usize())
            .enumerate()
            .filter_map(|(i, present)| if present { Some(i.into()) } else { None })
    }
}

impl Deref for SymbolBitMatrix {
    type Target = BitMatrix;
    fn deref(&self) -> &Self::Target {
        &self.bit_matrix
    }
}

impl DerefMut for SymbolBitMatrix {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bit_matrix
    }
}

static TRUE: bool = true;
static FALSE: bool = false;

impl ops::Index<(Symbol, Symbol)> for SymbolBitMatrix {
    type Output = bool;
    fn index(&self, index: (Symbol, Symbol)) -> &Self::Output {
        if self.bit_matrix[(index.0.usize(), index.1.usize())] {
            &TRUE
        } else {
            &FALSE
        }
    }
}
