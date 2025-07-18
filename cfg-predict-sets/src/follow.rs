//! FOLLOW sets.

use std::collections::BTreeMap;

use cfg_grammar::Cfg;

use crate::sets::PerSymbolSetVal;

use super::{PerSymbolSets, PredictSets};

/// FOLLOW sets.
pub struct FollowSets {
    /// Mapping from nonterminals to FOLLOW sets.
    map: PerSymbolSets,
}

impl FollowSets {
    /// Compute all FOLLOW sets of the grammar.
    /// Returns FollowSets.
    pub fn new(grammar: &Cfg, first_sets: &PerSymbolSets) -> Self {
        let mut this = FollowSets {
            map: BTreeMap::new(),
        };

        let mut roots = grammar.roots().to_vec();
        roots.sort();
        for rule in grammar.rules() {
            let follow_set = this
                .map
                .entry(rule.lhs)
                .or_insert_with(PerSymbolSetVal::new);
            if roots.binary_search(&rule.lhs).is_ok() {
                follow_set.has_none = true;
            }
        }

        let mut changed = true;
        while changed {
            changed = false;
            let terminal_set = grammar.terminal_symbols();
            for rule in grammar.rules() {
                let mut follow_set = this
                    .map
                    .get(&rule.lhs)
                    .cloned()
                    .expect("FOLLOW set not found");

                for &sym in rule.rhs.iter().rev() {
                    if terminal_set[sym] {
                        follow_set.clear();
                        follow_set.push(sym);
                    } else {
                        let followed = this.map.get_mut(&sym).unwrap();
                        let prev_cardinality = followed.len();
                        followed.extend(follow_set.iter().cloned());
                        followed.sort_unstable();
                        followed.dedup();
                        changed |= prev_cardinality != followed.len();

                        let first_set = first_sets.get(&sym).unwrap();
                        if !first_set.has_none {
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
