//! FIRST sets.

use cfg_grammar::RuleContainer;

use super::FirstSets;
use super::PerSymbolSets;
use super::PredictSets;

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
    pub fn new<G>(grammar: &G) -> Self
    where
        G: RuleContainer + Default,
    {
        let reversed_grammar = grammar.reverse();
        let map = {
            let first_sets = FirstSets::new(&reversed_grammar);
            // E0597: `reversed_grammar` does not live long enough
            //   label: borrowed value does not live long enough
            first_sets.map
        };
        LastSets { map }
    }
}

impl PredictSets for LastSets {
    /// Returns a reference to FIRST sets.
    fn predict_sets(&self) -> &PerSymbolSets {
        &self.map
    }
}
