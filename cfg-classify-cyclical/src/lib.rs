//! Cycle detection and elimination.

use std::collections::BTreeMap;

use cfg_grammar::{Cfg, CfgRule, RuleRef, SymbolBitSet};
use cfg_symbol::Symbol;
use cfg_symbol_bit_matrix::SymbolBitMatrix;

/// Provides information about cycles among unit derivations in the grammar. There are two ways of
/// pruning cycles.
pub struct Cycles<'a> {
    grammar: &'a mut Cfg,
    unit_derivation: SymbolBitMatrix,
    cycle_free: Option<bool>,
}

/// An iterator over the grammar's useless rules.
pub struct CycleParticipants<'a, I> {
    rules: I,
    cycles: &'a Cycles<'a>,
}

impl<'a> Cycles<'a> {
    /// Analyzes the grammar's cycles.
    pub fn new(grammar: &mut Cfg) -> Self {
        let unit_derivation = SymbolBitMatrix::unit_derivation_matrix(grammar);
        Cycles {
            unit_derivation,
            cycle_free: None,
            grammar,
        }
    }

    /// Checks whether the grammar is cycle-free.
    pub fn cycle_free(&mut self) -> bool {
        *self.cycle_free.get_or_insert_with(|| {
            (0..self.grammar.num_syms()).all(|i| !self.unit_derivation[(i.into(), i.into())])
        })
    }

    /// Iterates over rules that participate in a cycle.
    pub fn classify(&self) -> impl Iterator<Item = (RuleRef, bool)> + '_ {
        self.grammar.rules().map(move |rule| {
            (
                rule,
                rule.rhs.len() == 1 && self.unit_derivation[(rule.rhs[0], rule.lhs)],
            )
        })
    }

    /// Iterates over rules that participate in a cycle.
    pub fn cycle_participants(&self, get_cyclical: bool) -> impl Iterator<Item = RuleRef> + '_ {
        self.classify().filter_map(move |(rule, is_cyclical)| {
            if is_cyclical ^ !get_cyclical {
                Some(rule)
            } else {
                None
            }
        })
    }

    /// Removes all rules that participate in a cycle. Doesn't preserve the language represented
    /// by the grammar.
    pub fn remove_cycles(&mut self) {
        if !self.cycle_free() {
            self.grammar.retain(|rule| {
                !(rule.rhs.len() == 1 && self.unit_derivation[(rule.rhs[0], rule.lhs)])
            });
        }
    }

    /// Rewrites all rules that participate in a cycle. Preserves the language represented
    /// by the grammar.
    pub fn rewrite_cycles(&mut self) {
        let mut translation = BTreeMap::new();
        let mut row = SymbolBitSet::from_elem(self.grammar, false);
        if !self.cycle_free() {
            let unit_derivation = &self.unit_derivation;
            self.grammar.retain(|rule| {
                // We have `A ::= B`.
                let lhs = rule.lhs.into();
                if rule.rhs.len() == 1 && unit_derivation[(rule.rhs[0].into(), lhs)] {
                    // `B` derives `A`.
                    if !translation.contains_key(&rule.lhs) {
                        // Start rewrite. Check which symbols participate in this cycle.
                        // Get the union of `n`th row and column.
                        for (i, lhs_derives) in unit_derivation.iter_row(lhs.into()).enumerate() {
                            row.set(i.into(), lhs_derives && unit_derivation[(i.into(), lhs)])
                        }
                        for sym in row.iter() {
                            translation.insert(sym, Some(rule.lhs));
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
