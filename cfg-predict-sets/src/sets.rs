use std::collections::{BTreeMap, BTreeSet};

use cfg_symbol::Symbol;

struct PerSymbolSetKey {
    sym: Symbol,
    idx: usize,
    has_empty: bool,
}

/// The representation of FIRST and FOLLOW sets.
pub type PerSymbolSets = BTreeMap<PerSymbolSetKey, Symbol>;

pub trait PredictSets {
    fn predict_sets(&self) -> &PerSymbolSets;
}
