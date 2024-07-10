use bit_matrix::BitMatrix;

use cfg_grammar::RuleContainer;

/// Returns the direct derivation matrix.
pub fn direct_derivation_matrix<'a, G>(grammar: &'a G) -> BitMatrix
where
    G: RuleContainer,
{
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
pub fn reachability_matrix<'a, G>(grammar: &'a G) -> BitMatrix
where
    G: RuleContainer,
{
    let mut result = direct_derivation_matrix(grammar);
    result.transitive_closure();
    result.reflexive_closure();
    result
}

/// Returns the unit derivation matrix.
pub fn unit_derivation_matrix<'a, G>(grammar: &'a G) -> BitMatrix
where
    G: RuleContainer,
{
    let num_syms = grammar.num_syms();
    let mut unit_derivation = BitMatrix::new(num_syms, num_syms);

    for rule in grammar.rules() {
        // A rule of form `A ::= A` is not a cycle. We can represent unit rules in the form of
        // a directed graph. The rule `A ::= A` is then presented as a self-loop. Self-loops
        // aren't cycles.
        if rule.rhs.len() == 1 && rule.lhs != rule.rhs[0] {
            unit_derivation.set(rule.lhs.into(), rule.rhs[0].into(), true);
        }
    }

    unit_derivation.transitive_closure();
    unit_derivation
}
