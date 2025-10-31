//! Definitions for accessing the result of FIRST, FOLLOW
//! and LAST set computation.

use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};

use cfg_symbol::Symbol;

/// A set of symbols implemented as a list.
/// May contain a "none" value in case a nullable was found.
#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Debug)]
pub struct PerSymbolSetVal {
    /// Nullable found.
    pub has_none: bool,
    /// Set of symbols.
    pub list: Vec<Symbol>,
}

/// The representation of FIRST and FOLLOW sets.
pub type PerSymbolSets = BTreeMap<Symbol, PerSymbolSetVal>;

/// Trait for grabbing the result of FIRST, FOLLOW and LAST
/// set computation.
pub trait PredictSets {
    /// Provides access to the mapping from a symbol
    /// to a symbol set.
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

    /// Returns whether a nullable was found.
    pub fn has_none(&self) -> bool {
        self.has_none
    }

    pub(crate) fn len(&self) -> usize {
        self.list.len() + self.has_none as usize
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
