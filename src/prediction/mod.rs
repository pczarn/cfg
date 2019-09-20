//! Prediction for predictive parsers.

// Based on code by Niko Matsakis.

mod distance;
mod first;
mod last;
mod follow;

use std::collections::{BTreeMap, BTreeSet};

use symbol::Symbol;

pub use self::distance::MinimalDistance;
pub use self::first::FirstSets;
pub use self::last::LastSets;
pub use self::follow::FollowSets;

/// The representation of FIRST and FOLLOW sets.
pub type PerSymbolSets = BTreeMap<Symbol, BTreeSet<Option<Symbol>>>;
