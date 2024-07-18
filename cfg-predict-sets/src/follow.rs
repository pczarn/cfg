//! FOLLOW sets.

use std::collections::{BTreeMap, BTreeSet};

use cfg_symbol::Symbol;

use cfg_grammar::symbol::symbol_set::SymbolBitSet;
use cfg_grammar::RuleContainer;

use super::{PerSymbolSets, PredictSets};

/// FOLLOW sets.
pub struct FollowSets {
    /// Mapping from nonterminals to FOLLOW sets.
    map: PerSymbolSets,
}

impl FollowSets {
    /// Compute all FOLLOW sets of the grammar.
    /// Returns FollowSets.
    pub fn new<'a, G>(grammar: &'a G, start_sym: Symbol, first_sets: &PerSymbolSets) -> Self
    where
        G: RuleContainer,
    {
        let mut this = FollowSets {
            map: BTreeMap::new(),
        };

        for rule in grammar.rules() {
            let follow_set = this.map.entry(rule.lhs).or_insert_with(BTreeSet::new);
            if rule.lhs == start_sym {
                follow_set.insert(None);
            }
        }

        let mut changed = true;
        while changed {
            changed = false;
            let terminal_set = SymbolBitSet::terminal_set(grammar);
            for rule in grammar.rules() {
                let mut follow_set = this.map.get(&rule.lhs).unwrap().clone();

                for &sym in rule.rhs.iter().rev() {
                    if terminal_set.has_sym(sym) {
                        follow_set.clear();
                        follow_set.insert(Some(sym));
                    } else {
                        let followed = this.map.get_mut(&sym).unwrap();
                        let prev_cardinality = followed.len();
                        followed.extend(follow_set.iter().cloned());
                        changed |= prev_cardinality != followed.len();

                        let first_set = first_sets.get(&sym).unwrap();
                        if !first_set.contains(&None) {
                            follow_set.clear();
                        }
                        follow_set.extend(first_set.iter().cloned());
                    }
                }
            }
        }

        this
    }
}

impl PredictSets for FollowSets {
    /// Returns a reference to FIRST sets.
    fn predict_sets(&self) -> &PerSymbolSets {
        &self.map
    }
}
