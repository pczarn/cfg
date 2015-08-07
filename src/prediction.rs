//! Prediction for predictive parsers.

use std::collections::{BTreeMap, BTreeSet};

use grammar::{ContextFree, ContextFreeRef};
use rule::GrammarRule;
use symbol::{SymbolSource, GrammarSymbol, TerminalSymbolSet};

/// The representation of FIRST and FOLLOW sets.
pub type PerSymbolSets<S> = BTreeMap<S, BTreeSet<Option<S>>>;

/// FIRST sets.
pub struct FirstSets<S> where S: GrammarSymbol {
    map: PerSymbolSets<S>,
}

/// FOLLOW sets.
pub struct FollowSets<S> where S: GrammarSymbol {
    /// Mapping from nonterminals to FOLLOW sets.
    map: PerSymbolSets<S>,
}

// Based on code by Niko Matsakis.
impl<S> FirstSets<S> where S: GrammarSymbol {
    /// Compute all FIRST sets of the grammar.
    ///
    /// We define a binary relation FIRST(N, S), in which N is related to S
    /// if the grammar has a production of the form `N ⸬= α S β`, where
    /// α is a nullable string of symbols.
    ///
    /// We compute the transitive closure of this relation.
    pub fn new<'a, G>(grammar: &'a G) -> Self where
                G: ContextFree<Symbol=S> + TerminalSymbolSet,
                &'a G: ContextFreeRef<'a, Target=G> {
        let mut this = FirstSets {
            map: BTreeMap::new(),
        };

        let mut lookahead = vec![];
        let mut changed = true;
        while changed {
            changed = false;
            for rule in grammar.rules() {
                this.first_set_collect(grammar, &mut lookahead, rule.rhs());
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
    pub fn first_sets(&self) -> &PerSymbolSets<S> {
        &self.map
    }

    /// Compute a FIRST set.
    fn first_set_collect<G>(&self, grammar: &G, vec: &mut Vec<Option<S>>, rhs: &[S]) where
                G: ContextFree<Symbol=S> + TerminalSymbolSet {
        for &sym in rhs {
            let mut nullable = false;
            if grammar.is_terminal(sym) {
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

impl<S> FollowSets<S> where S: GrammarSymbol {
    /// Compute all FOLLOW sets of the grammar.
    /// Returns FollowSets.
    pub fn new<'a, G>(grammar: &'a G, first_sets: &FirstSets<S>) -> Self
            where G: ContextFree<Symbol=S> + TerminalSymbolSet,
                  &'a G: ContextFreeRef<'a, Target=G> {
        let mut this = FollowSets {
            map: BTreeMap::new()
        };

        for rule in grammar.rules() {
            let follow_set = this.map.entry(rule.lhs()).or_insert_with(|| BTreeSet::new());
            if rule.lhs() == grammar.start_sym() {
                follow_set.insert(None);
            }
        }

        let mut changed = true;
        while changed {
            changed = false;
            for rule in grammar.rules() {
                let mut follow_set = this.map.get(&rule.lhs()).unwrap().clone();

                for &sym in rule.rhs().iter().rev() {
                    if grammar.is_terminal(sym) {
                        follow_set.clear();
                        follow_set.insert(Some(sym));
                    } else {
                        let followed = this.map.get_mut(&sym).unwrap();
                        let prev_cardinality = followed.len();
                        followed.extend(follow_set.iter().cloned());
                        changed |= prev_cardinality != followed.len();

                        let first_set = first_sets.map.get(&sym).unwrap();
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

    /// Returns a reference to FOLLOW sets.
    pub fn follow_sets(&self) -> &PerSymbolSets<S> {
        &self.map
    }
}
