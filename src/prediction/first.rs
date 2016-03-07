//! FIRST sets.

use std::collections::{BTreeMap, BTreeSet};

use grammar::{ContextFree, ContextFreeRef};
use prediction::PerSymbolSets;
use rule::GrammarRule;
use rule::terminal_set::TerminalSet;
use symbol::Symbol;

/// FIRST sets.
pub struct FirstSets {
    map: PerSymbolSets,
}

impl FirstSets {
    /// Compute all FIRST sets of the grammar.
    ///
    /// We define a binary relation FIRST(N, S), in which N is related to S
    /// if the grammar has a production of the form `N ⸬= α S β`, where
    /// α is a nullable string of symbols.
    ///
    /// We compute the transitive closure of this relation.
    pub fn new<'a, G>(grammar: &'a G) -> Self
        where G::TerminalSet: TerminalSet,
              G: ContextFree,
              &'a G: ContextFreeRef<'a, Target = G>
    {
        let mut this = FirstSets { map: BTreeMap::new() };

        let mut lookahead = vec![];
        let mut changed = true;
        while changed {
            changed = false;
            let terminal_set = grammar.terminal_set();
            for rule in grammar.rules() {
                this.first_set_collect(&terminal_set, &mut lookahead, rule.rhs());
                let first_set = this.map.entry(rule.lhs()).or_insert_with(|| BTreeSet::new());
                let prev_cardinality = first_set.len();
                first_set.extend(lookahead.iter().cloned());
                lookahead.clear();
                changed |= first_set.len() != prev_cardinality;
            }
        }

        this
    }

    /// Returns a reference to FIRST sets.
    pub fn first_sets(&self) -> &PerSymbolSets {
        &self.map
    }

    /// Compute a FIRST set.
    fn first_set_collect<T>(&self, terminal_set: &T, vec: &mut Vec<Option<Symbol>>, rhs: &[Symbol])
        where T: TerminalSet
    {
        for &sym in rhs {
            let mut nullable = false;
            if terminal_set.has_sym(sym) {
                vec.push(Some(sym));
            } else {
                match self.map.get(&sym) {
                    None => {
                        // This should only happen during set construction; it
                        // corresponds to an entry that has not yet been
                        // built. Otherwise, it would mean a nonterminal with
                        // no productions. Either way, the resulting first set
                        // should be empty.
                    }
                    Some(set) => {
                        for &opt_terminal in set {
                            if opt_terminal.is_some() {
                                vec.push(opt_terminal);
                            } else {
                                nullable = true;
                            }
                        }
                    }
                }
            }
            if !nullable {
                return;
            }
        }
        vec.push(None);
    }
}
