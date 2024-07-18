use bit_matrix::BitMatrix;

use crate::local_prelude::*;

/// Returns the direct derivation matrix.
pub fn direct_derivation_matrix<'a>(grammar: &'a Cfg) -> BitMatrix {
    let num_syms = grammar.sym_source().num_syms();
    let mut derivation = BitMatrix::new(num_syms, num_syms);

    for rule in grammar.rules() {
        derivation.set(rule.lhs.usize(), rule.lhs.usize(), true);
        for &sym in rule.rhs {
            derivation.set(rule.lhs.usize(), sym.usize(), true);
        }
    }
    derivation
}

/// Returns the derivation matrix.
pub fn reachability_matrix<'a>(grammar: &'a Cfg) -> BitMatrix {
    let mut result = direct_derivation_matrix(grammar);
    result.transitive_closure();
    result.reflexive_closure();
    result
}
