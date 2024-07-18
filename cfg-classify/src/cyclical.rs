//! Cycle detection and elimination.

use std::collections::BTreeMap;

use bit_matrix::BitMatrix;
use bit_vec::BitVec;

use cfg_grammar::{
    rule::{cfg_rule::CfgRule, RuleRef},
    AsRuleRef, RuleContainer,
};
use cfg_symbol::Symbol;

use crate::derivation;

/// Provides information about cycles among unit derivations in the grammar. There are two ways of
/// pruning cycles.
pub struct Cycles<'a> {
    grammar: &'a Cfg,
    unit_derivation: BitMatrix,
    cycle_free: bool,
}

/// An iterator over the grammar's useless rules.
pub struct CycleParticipants<'a, I> {
    rules: I,
    cycles: &'a Cycles<'a>,
}

impl<'a> Cycles<'a> {
    /// Analyzes the grammar's cycles.
    pub fn new<'a>(grammar: &'a mut G) -> Cycles<&'a mut G> {
        let unit_derivation = derivation::unit_derivation_matrix(grammar);
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
    G: RuleContainer,
{
    /// Iterates over rules that participate in a cycle.
    pub fn cycle_participants(
        &'a self,
    ) -> CycleParticipants<'a, G, impl Iterator<Item = RuleRef<'a>>> {
        CycleParticipants {
            rules: self.grammar.rules(),
            cycles: self,
        }
    }

    /// Removes all rules that participate in a cycle. Doesn't preserve the language represented
    /// by the grammar.
    pub fn remove_cycles(&mut self) {
        if !self.cycle_free {
            let unit_derivation = &self.unit_derivation;
            self.grammar.retain(|rule| {
                rule.rhs.len() != 1 || !unit_derivation[(rule.rhs[0].into(), rule.lhs.into())]
            });
        }
    }

    /// Rewrites all rules that participate in a cycle. Preserves the language represented
    /// by the grammar.
    pub fn rewrite_cycles(&mut self) {
        let mut translation = BTreeMap::new();
        let mut row = BitVec::from_elem(self.grammar.num_syms(), false);
        if !self.cycle_free {
            let unit_derivation = &self.unit_derivation;
            self.grammar.retain(|rule| {
                // We have `A ::= B`.
                let lhs = rule.lhs.into();
                if rule.rhs.len() == 1 && unit_derivation[(rule.rhs[0].into(), lhs)] {
                    // `B` derives `A`.
                    if !translation.contains_key(&rule.lhs) {
                        // Start rewrite. Check which symbols participate in this cycle.
                        // Get the union of `n`th row and column.
                        for (i, lhs_derives) in unit_derivation.iter_row(lhs).enumerate() {
                            row.set(i, lhs_derives && unit_derivation[(i, lhs)])
                        }
                        for (i, is_in_cycle) in row.iter().enumerate() {
                            if is_in_cycle {
                                translation.insert(Symbol::from(i), Some(rule.lhs));
                            }
                        }
                        translation.insert(rule.lhs, None);
                    }
                    false
                } else {
                    true
                }
            });
            // Rewrite symbols using the `translation` map, potentially leaving
            // some symbols unused.
            let mut rewritten_rules = vec![];
            self.grammar.retain(|mut rule| {
                let mut changed = false;
                if let Some(&Some(new_lhs)) = translation.get(&rule.lhs) {
                    rule.lhs = new_lhs;
                    changed = true;
                }
                let mut rhs = rule.rhs.to_vec();
                for sym in &mut rhs {
                    if let Some(&Some(new_sym)) = translation.get(sym) {
                        *sym = new_sym;
                        changed = true;
                    }
                }
                if changed {
                    rewritten_rules.push(CfgRule {
                        lhs: rule.lhs,
                        rhs,
                        history_id: rule.history_id,
                    });
                }
                !changed
            });
            for rule in rewritten_rules {
                self.grammar.add_rule(rule.as_rule_ref());
            }
        }
    }
}

impl<'a, G, I> Iterator for CycleParticipants<'a, G, I>
where
    G: RuleContainer + 'a,
    I: Iterator<Item = RuleRef<'a>>,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cycles.cycle_free {
            return None;
        }

        for rule in &mut self.rules {
            if rule.rhs.len() == 1
                && self.cycles.unit_derivation[(rule.rhs[0].into(), rule.lhs.into())]
            {
                return Some(rule);
            }
        }

        None
    }
}
