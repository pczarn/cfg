//! Cycle detection and elimination.

use std::collections::BTreeMap;

use bit_matrix::{FixedBitVec, FixedBitMatrix};

use grammar::{ContextFree, ContextFreeRef, ContextFreeMut};
use rule::GrammarRule;
use symbol::{SymbolSource, GrammarSymbol};

/// Provides information about cycles among unit derivations in the grammar. There are two ways of
/// pruning cycles.
pub struct Cycles<G> {
    grammar: G,
    unit_derivation: FixedBitMatrix,
    cycle_free: bool,
}

/// An iterator over the grammar's useless rules.
pub struct CycleParticipants<'a, G: 'a, R> {
    rules: R,
    cycles: &'a Cycles<&'a mut G>,
}

/// Returns the unit derivation matrix.
fn unit_derivation_matrix<'a, G>(grammar: &'a G) -> FixedBitMatrix where
            G: ContextFree,
            &'a G: ContextFreeRef<'a, Target=G> {
    let num_syms = grammar.sym_source().num_syms();
    let mut unit_derivation = FixedBitMatrix::new(num_syms, num_syms);

    for rule in grammar.rules() {
        // A rule of form `A ::= A` is not a cycle. We can represent unit rules in the form of
        // a directed graph. The rule `A ::= A` is then presented as a self-loop. Self-loops
        // aren't cycles.
        if rule.rhs().len() == 1 && rule.lhs() != rule.rhs()[0] {
            unit_derivation.set(rule.lhs().usize(), rule.rhs()[0].usize(), true);
        }
    }

    unit_derivation.transitive_closure();
    unit_derivation
}

impl<'a, G> Cycles<&'a mut G> where
            G: ContextFree,
            for<'b> &'b G: ContextFreeRef<'b, Target=G>,
            for<'b> &'b mut G: ContextFreeMut<'b, Target=G> {
    /// Analyzes the grammar's cycles.
    pub fn new(grammar: &'a mut G) -> Cycles<&'a mut G> {
        let unit_derivation = unit_derivation_matrix(grammar);
        let cycle_free = (0 .. grammar.num_syms()).all(|i| !unit_derivation[(i, i)]);
        Cycles {
            unit_derivation: unit_derivation,
            cycle_free: cycle_free,
            grammar: grammar,
        }
    }

    /// Checks whether the grammar is cycle-free.
    pub fn cycle_free(&self) -> bool {
        self.cycle_free
    }
}

impl<'a, G> Cycles<&'a mut G> where
        G: ContextFree,
        &'a G: ContextFreeRef<'a, Target=G>,
        &'a mut G: ContextFreeMut<'a, Target=G> {
    /// Iterates over rules that participate in a cycle.
    pub fn cycle_participants(&'a self)
                -> CycleParticipants<'a, G, <&'a G as ContextFreeRef<'a>>::Rules> {
        CycleParticipants {
            rules: self.grammar.rules(),
            cycles: self,
        }
    }

    /// Removes all rules that participate in a cycle. Doesn't preserve the language represented
    /// by the grammar.
    pub fn remove_cycles(&mut self) where
                &'a G: ContextFreeRef<'a, Target=G>,
                &'a mut G: ContextFreeMut<'a, Target=G> {
        if !self.cycle_free {
            let unit_derivation = &self.unit_derivation;
            self.grammar.retain(|lhs, rhs, _| {
                rhs.len() != 1 || !unit_derivation[(rhs[0].usize(), lhs.usize())]
            });
        }
    }

    /// Rewrites all rules that participate in a cycle. Preserves the language represented
    /// by the grammar.
    pub fn rewrite_cycles(&mut self) where
                G::History: Clone,
                &'a G: ContextFreeRef<'a, Target=G>,
                &'a mut G: ContextFreeMut<'a, Target=G> {
        let mut translation = BTreeMap::new();
        let mut row = FixedBitVec::from_elem(self.grammar.num_syms(), false);
        if !self.cycle_free {
            let unit_derivation = &self.unit_derivation;
            self.grammar.retain(|lhs_sym, rhs, _| {
                // We have `A ::= B`.
                let lhs = lhs_sym.usize();
                if rhs.len() == 1 && unit_derivation[(rhs[0].usize(), lhs)] {
                    // `B` derives `A`.
                    if !translation.contains_key(&lhs_sym) {
                        // Start rewrite. Check which symbols participate in this cycle.
                        // Get the union of `n`th row and column.
                        for (i, lhs_derives) in unit_derivation.iter_row(lhs).enumerate() {
                            row.set(i, lhs_derives && unit_derivation[(i, lhs)])
                        }
                        for (i, is_in_cycle) in row.iter().enumerate() {
                            if is_in_cycle {
                                translation.insert(G::Symbol::from(i as u64), Some(lhs_sym));
                            }
                        }
                        translation.insert(lhs_sym, None);
                    }
                    false
                } else {
                    true
                }
            });
            // Rewrite symbols using the `trnslation` map, potentially leaving
            // some symbols unused.
            let mut rewritten_rules = vec![];
            self.grammar.retain(|mut lhs, rhs, history| {
                let mut changed = false;
                if let Some(&Some(new_lhs)) = translation.get(&lhs) {
                    lhs = new_lhs;
                    changed = true;
                }
                let mut rhs = rhs.to_vec();
                for sym in rhs.iter_mut() {
                    if let Some(&Some(new_sym)) = translation.get(sym) {
                        *sym = new_sym;
                        changed = true;
                    }
                }
                if changed {
                    rewritten_rules.push((lhs, rhs, history.clone()));
                }
                !changed
            });
            for (lhs, rhs, history) in rewritten_rules {
                self.grammar.add_rule(lhs, &rhs[..], history);
            }
        }
    }
}

impl<'a, G> Iterator for CycleParticipants<'a, G, <&'a G as ContextFreeRef<'a>>::Rules>
        where
            G: ContextFree + 'a,
            &'a G: ContextFreeRef<'a, Target=G> {
    type Item = <<&'a G as ContextFreeRef<'a>>::Rules as Iterator>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cycles.cycle_free {
            return None;
        }

        while let Some(rule) = self.rules.next() {
            if rule.rhs().len() == 1 && self.cycles.unit_derivation[(rule.rhs()[0].usize(),
                                                                     rule.lhs().usize())] {
                return Some(rule);
            }
        }

        None
    }
}
