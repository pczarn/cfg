//! FIRST sets.

use std::collections::BTreeMap;

use cfg_grammar::{Cfg, SymbolBitSet};
use cfg_symbol::Symbol;

use crate::sets::PerSymbolSetVal;

use super::{PerSymbolSets, PredictSets};

/// Collector of FIRST sets.
pub struct FirstSets {
    pub(super) map: PerSymbolSets,
    lookahead: PerSymbolSetVal,
    changed: bool,
    terminal_set: SymbolBitSet,
}

// struct FirstSets;

// impl FirstSets {
//     fn new(grammar: &Cfg) -> Self {
//         FirstSets2
//     }
// }

impl FirstSets {
    /// Compute all FIRST sets of the grammar.
    ///
    /// We define a binary relation FIRST(N, S), in which N is related to S
    /// if the grammar has a production of the form `N ⸬= α S β`, where
    /// α is a nullable string of symbols.
    ///
    /// We compute the transitive closure of this relation.
    pub fn new(grammar: &Cfg) -> Self {
        let mut this = FirstSets {
            map: BTreeMap::new(),
            lookahead: PerSymbolSetVal::new(),
            changed: true,
            terminal_set: grammar.terminal_symbols(),
        };

        this.collect_from(grammar);
        this
    }

    /// Calculates a FIRST set for a string of symbols.
    pub fn first_set_for_string(&self, string: &[Symbol]) -> PerSymbolSetVal {
        let mut result = vec![];
        for &sym in string {
            if self.terminal_set[sym] {
                result.push(sym);
                break;
            } else {
                let first_set = self
                    .map
                    .get(&sym)
                    .expect("FIRST set not found in PerSymbolSets");
                result.clone_from(&first_set.list);
                if !result.is_empty() {
                    break;
                }
            }
        }
        PerSymbolSetVal {
            has_none: result.is_empty(),
            list: result,
        }
    }

    fn collect_from(&mut self, grammar: &Cfg) {
        while self.changed {
            self.changed = false;
            for rule in grammar.rules() {
                let set_changed = self.process_rule(rule.lhs, &rule.rhs[..]);
                self.changed |= set_changed;
            }
        }
    }

    fn process_rule(&mut self, lhs: Symbol, rhs: &[Symbol]) -> bool {
        self.first_set_collect(rhs);
        let first_set = self.map.entry(lhs).or_insert_with(PerSymbolSetVal::new);
        let prev_cardinality = first_set.len();
        first_set.list.extend(self.lookahead.list.iter().cloned());
        first_set.list.sort_unstable();
        first_set.list.dedup();
        first_set.has_none |= self.lookahead.has_none;
        self.lookahead.list.clear();
        self.lookahead.has_none = false;
        prev_cardinality != first_set.len()
    }

    /// Compute a FIRST set.
    fn first_set_collect(&mut self, rhs: &[Symbol]) {
        for &sym in rhs {
            let mut nullable = false;
            if self.terminal_set[sym] {
                self.lookahead.list.push(sym);
                self.lookahead.list.sort_unstable();
                self.lookahead.list.dedup();
            } else {
                match self.map.get(&sym) {
                    None => {
                        // This should only happen during set construction; it
                        // corresponds to an entry that has not yet been
                        // built. Otherwise, it would mean a nonterminal with
                        // no productions. Either way, the resulting first set
                        // should be empty.
                    }
                    Some(&PerSymbolSetVal { has_none, ref list }) => {
                        self.lookahead.list.extend(list.iter().copied());
                        self.lookahead.list.sort_unstable();
                        self.lookahead.list.dedup();
                        if has_none {
                            nullable = true;
                        }
                    }
                }
            }
            if !nullable {
                // Successfully found a FIRST symbol.
                return;
            }
        }
        self.lookahead.has_none = true;
    }
}

impl PredictSets for FirstSets {
    /// Returns a reference to FIRST sets.
    fn predict_sets(&self) -> &PerSymbolSets {
        &self.map
    }
}
