//! FIRST sets.

use crate::prelude::*;

use super::FirstSetsCollector;
use super::PerSymbolSets;

/// FIRST sets.
pub struct LastSets {
    map: PerSymbolSets,
}

impl LastSets {
    /// Compute all LAST sets of the grammar.
    ///
    /// We define a binary relation LAST(N, S), in which N is related to S
    /// if the grammar has a production of the form `N ⸬= α S β`, where
    /// β is a nullable string of symbols.
    ///
    /// We compute the transitive closure of this relation.
    pub fn new<'a, G>(grammar: &'a G) -> Self
    where
        G: RuleContainer + Default,
        for<'b> &'b G: RuleContainerRef<'b, Target = G>,
    {
        let reversed_grammar = grammar.reverse();
        let map = {
            let first_sets = FirstSetsCollector::new(&reversed_grammar);
            // E0597: `reversed_grammar` does not live long enough
            //   label: borrowed value does not live long enough
            first_sets.map
        };
        LastSets { map }
    }

    /// Returns a reference to FIRST sets.
    pub fn last_sets(&self) -> &PerSymbolSets {
        &self.map
    }
}
