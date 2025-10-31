//! Extension trait for computing FIRST and FOLLOW sets.

use cfg_grammar::Cfg;

use crate::{FirstSets, FollowSets, PredictSets};

/// Extension trait for `Cfg`.
pub trait CfgSetsExt {
    /// Computes the FIRST sets for this grammar.
    fn first_sets(&self) -> FirstSets;
    /// Computes the FOLLOW sets for this grammar.
    fn follow_sets(&self) -> FollowSets;
    /// Computes the FOLLOW sets for this grammar with the
    /// given FIRST sets.
    fn follow_sets_with_first(&self, first_sets: &FirstSets) -> FollowSets;
}

impl CfgSetsExt for Cfg {
    fn first_sets(&self) -> FirstSets {
        FirstSets::new(self)
    }

    fn follow_sets(&self) -> FollowSets {
        FollowSets::new(self, self.first_sets().predict_sets())
    }

    fn follow_sets_with_first(&self, first_sets: &FirstSets) -> FollowSets {
        FollowSets::new(self, first_sets.predict_sets())
    }
}
