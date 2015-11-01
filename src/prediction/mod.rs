//! Prediction for predictive parsers.

mod first;
mod follow;

use std::collections::{BTreeMap, BTreeSet};

pub use self::first::FirstSets;
pub use self::follow::FollowSets;

/// The representation of FIRST and FOLLOW sets.
pub type PerSymbolSets<S> = BTreeMap<S, BTreeSet<Option<S>>>;
