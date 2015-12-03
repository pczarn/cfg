//! Prediction for predictive parsers.

// Based on code by Niko Matsakis.

mod first;
mod follow;

use std::collections::{BTreeMap, BTreeSet};

pub use self::first::FirstSets;
pub use self::follow::FollowSets;

/// The representation of FIRST and FOLLOW sets.
pub type PerSymbolSets<S> = BTreeMap<S, BTreeSet<Option<S>>>;
