use std::collections::{BTreeMap, BTreeSet};

use cfg_symbol::Symbol;

/// The representation of FIRST and FOLLOW sets.
pub type PerSymbolSets = BTreeMap<Symbol, BTreeSet<Option<Symbol>>>;

pub trait PredictSets {
    fn predict_sets(&self) -> &PerSymbolSets;
}
