use bit_matrix::BitMatrix;

use cfg_grammar::Cfg;

struct SymbolBitMatrix {
    bit_matrix: BitMatrix,
}

impl SymbolBitMatrix {
    pub fn new(num_syms: usize) -> Self {
        SymbolBitMatrix {
            bit_matrix: BitMatrix::new(num_syms, num_syms),
        }
    }
}

/// Returns the direct derivation matrix.
pub fn direct_derivation_matrix<'a>(grammar: &'a Cfg) -> BitMatrix {
    let mut derivation = SymbolBitMatrix::new(grammar.sym_source().num_syms());

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
