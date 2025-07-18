use cfg_grammar::Cfg;

use crate::{FirstSets, FollowSets, PredictSets};

pub trait CfgSetsExt {
    fn first_sets(&self) -> FirstSets;
    fn follow_sets(&self) -> FollowSets;
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
