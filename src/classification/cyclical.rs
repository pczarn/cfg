//! Cycle detection and elimination.

use std::collections::BTreeMap;

use bit_matrix::BitMatrix;
use bit_vec::BitVec;

use crate::analysis;
use crate::prelude::*;

/// Provides information about cycles among unit derivations in the grammar. There are two ways of
/// pruning cycles.
pub struct Cycles<G> {
    grammar: G,
    unit_derivation: BitMatrix,
    cycle_free: bool,
}

/// An iterator over the grammar's useless rules.
pub struct CycleParticipants<'a, G: 'a, R> {
    rules: R,
    cycles: &'a Cycles<&'a mut G>,
}

impl<'a, G> Cycles<&'a mut G>
where
    G: RuleContainer + Default,
    for<'b> &'b G: RuleContainerRef<'b, Target = G>,
    for<'b> &'b mut G: RuleContainerMut<'b, Target = G>,
{
    /// Analyzes the grammar's cycles.
    pub fn new(grammar: &'a mut G) -> Cycles<&'a mut G> {
        let unit_derivation = analysis::unit_derivation_matrix(grammar);
        let cycle_free = (0..grammar.num_syms()).all(|i| !unit_derivation[(i, i)]);
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

impl<'a, G> Cycles<&'a mut G>
where
    G: RuleContainer + Default,
    &'a G: RuleContainerRef<'a, Target = G>,
    &'a mut G: RuleContainerMut<'a, Target = G>,
{
    /// Iterates over rules that participate in a cycle.
    pub fn cycle_participants(
        &'a self,
    ) -> CycleParticipants<'a, G, <&'a G as RuleContainerRef<'a>>::Rules> {
        CycleParticipants {
            rules: self.grammar.rules(),
            cycles: self,
        }
    }

    /// Removes all rules that participate in a cycle. Doesn't preserve the language represented
    /// by the grammar.
    pub fn remove_cycles(&mut self)
    where
        &'a G: RuleContainerRef<'a, Target = G>,
        &'a mut G: RuleContainerMut<'a, Target = G>,
    {
        if !self.cycle_free {
            let unit_derivation = &self.unit_derivation;
            self.grammar.retain(|lhs, rhs, _| {
                rhs.len() != 1 || !unit_derivation[(rhs[0].into(), lhs.into())]
            });
        }
    }

    /// Rewrites all rules that participate in a cycle. Preserves the language represented
    /// by the grammar.
    pub fn rewrite_cycles(&mut self)
    where
        &'a G: RuleContainerRef<'a, Target = G>,
        &'a mut G: RuleContainerMut<'a, Target = G>,
    {
        let mut translation = BTreeMap::new();
        let mut row = BitVec::from_elem(self.grammar.num_syms(), false);
        if !self.cycle_free {
            let unit_derivation = &self.unit_derivation;
            self.grammar.retain(|lhs_sym, rhs, _| {
                // We have `A ::= B`.
                let lhs = lhs_sym.into();
                if rhs.len() == 1 && unit_derivation[(rhs[0].into(), lhs)] {
                    // `B` derives `A`.
                    if !translation.contains_key(&lhs_sym) {
                        // Start rewrite. Check which symbols participate in this cycle.
                        // Get the union of `n`th row and column.
                        for (i, lhs_derives) in unit_derivation.iter_row(lhs).enumerate() {
                            row.set(i, lhs_derives && unit_derivation[(i, lhs)])
                        }
                        for (i, is_in_cycle) in row.iter().enumerate() {
                            if is_in_cycle {
                                translation.insert(Symbol::from(i), Some(lhs_sym));
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
                for sym in &mut rhs {
                    if let Some(&Some(new_sym)) = translation.get(sym) {
                        *sym = new_sym;
                        changed = true;
                    }
                }
                if changed {
                    rewritten_rules.push((lhs, rhs, history));
                }
                !changed
            });
            for (lhs, rhs, history) in rewritten_rules {
                self.grammar.add_rule(lhs, &rhs[..], history);
            }
        }
    }
}

impl<'a, G> Iterator for CycleParticipants<'a, G, <&'a G as RuleContainerRef<'a>>::Rules>
where
    G: RuleContainer + Default + 'a,
    &'a G: RuleContainerRef<'a, Target = G>,
{
    type Item = <<&'a G as RuleContainerRef<'a>>::Rules as Iterator>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cycles.cycle_free {
            return None;
        }

        for rule in &mut self.rules {
            if rule.rhs().len() == 1
                && self.cycles.unit_derivation[(rule.rhs()[0].into(), rule.lhs().into())]
            {
                return Some(rule);
            }
        }

        None
    }
}
