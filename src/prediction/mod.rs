//! Prediction for predictive parsers.

// Based on code by Niko Matsakis.

mod distance;
mod first;
mod follow;
mod last;

use std::collections::{BTreeMap, BTreeSet};

use symbol::Symbol;

pub use self::distance::MinimalDistance;
pub use self::first::{FirstSets, FirstSetsCollector};
pub use self::follow::FollowSets;
pub use self::last::LastSets;

/// The representation of FIRST and FOLLOW sets.
pub type PerSymbolSets = BTreeMap<Symbol, BTreeSet<Option<Symbol>>>;
