use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};

use cfg_symbol::Symbol;

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct PerSymbolSetVal {
    pub(crate) has_none: bool,
    pub(crate) list: Vec<Symbol>,
}

/// The representation of FIRST and FOLLOW sets.
pub type PerSymbolSets = BTreeMap<Symbol, PerSymbolSetVal>;

pub trait PredictSets {
    fn predict_sets(&self) -> &PerSymbolSets;
}

impl PerSymbolSetVal {
    pub(crate) fn new() -> Self {
        PerSymbolSetVal {
            has_none: false,
            list: vec![],
        }
    }

    pub(crate) fn clear(&mut self) {
        self.list.clear();
        self.has_none = false;
    }
}

impl Deref for PerSymbolSetVal {
    type Target = Vec<Symbol>;
    fn deref(&self) -> &Self::Target {
        &self.list
    }
}

impl DerefMut for PerSymbolSetVal {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.list
    }
}
