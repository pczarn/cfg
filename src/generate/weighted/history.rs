use num::one;

use crate::{
    earley::history::{BuildHistory, History},
    history::{Binarize, HistorySource, RewriteSequence},
    GrammarRule, Symbol,
};

use super::Weight;

#[derive(Clone)]
pub struct WeightedHistory<W, H = History> {
    inherit: H,
    weight: W,
}

#[derive(Clone)]
pub struct WeightedSequenceHistory<W, H = History> {
    inherit: H,
    weight_one: W,
    weight_more: W,
}

/// Default history.
#[derive(Copy, Clone)]
pub struct BuildWeightedHistory {
    inherit: BuildHistory,
}

/// Default history.
#[derive(Copy, Clone)]
pub struct BuildWeightedSequenceHistory {
    inherit: BuildHistory,
}

impl<W: Weight, H> WeightedHistory<W, H> {
    pub fn with_history_and_weight(history: H, weight: W) -> Self {
        WeightedHistory {
            inherit: history,
            weight,
        }
    }

    pub fn weight(&self) -> W {
        self.weight
    }
}

impl<W: Weight, H> WeightedSequenceHistory<W, H> {
    pub fn with_history_and_weights(history: H, weight_one: W, weight_more: W) -> Self {
        WeightedSequenceHistory {
            inherit: history,
            weight_one,
            weight_more,
        }
    }

    pub fn weight_one(&self) -> W {
        self.weight_one
    }

    pub fn weight_more(&self) -> W {
        self.weight_more
    }
}

impl BuildWeightedHistory {
    /// Creates default history.
    pub(super) fn new(num_rules: usize) -> Self {
        Self {
            inherit: BuildHistory::new(num_rules),
        }
    }
}

impl BuildWeightedSequenceHistory {
    /// Creates default history.
    pub(super) fn new(num_rules: usize) -> Self {
        Self {
            inherit: BuildHistory::new(num_rules),
        }
    }
}

impl<W: Weight> HistorySource<WeightedHistory<W>> for BuildWeightedHistory {
    fn build(&mut self, lhs: Symbol, rhs: &[Symbol]) -> WeightedHistory<W> {
        WeightedHistory {
            inherit: self.inherit.build(lhs, rhs),
            weight: one(),
        }
    }
}

impl<W: Weight> HistorySource<WeightedSequenceHistory<W>> for BuildWeightedSequenceHistory {
    fn build(&mut self, lhs: Symbol, rhs: &[Symbol]) -> WeightedSequenceHistory<W> {
        WeightedSequenceHistory {
            inherit: self.inherit.build(lhs, rhs),
            weight_one: one(),
            weight_more: one(),
        }
    }
}

impl<W: Weight, H: RewriteSequence<Rewritten = H>> RewriteSequence
    for WeightedSequenceHistory<W, H>
{
    type Rewritten = WeightedHistory<W, H>;

    fn top(&self, rhs: Symbol, sep: Option<Symbol>, new_rhs: &[Symbol]) -> Self::Rewritten {
        let weight = if new_rhs.len() <= 1 {
            self.weight_one
        } else {
            self.weight_more
        };
        WeightedHistory {
            inherit: self.inherit.top(rhs, sep, new_rhs),
            weight,
        }
    }

    fn bottom(&self, rhs: Symbol, sep: Option<Symbol>, new_rhs: &[Symbol]) -> Self::Rewritten {
        let weight = if new_rhs.len() <= 1 {
            self.weight_one
        } else {
            self.weight_more
        };
        WeightedHistory {
            inherit: self.inherit.bottom(rhs, sep, new_rhs),
            weight,
        }
    }
}

impl<W: Weight, H: Binarize> Binarize for WeightedHistory<W, H> {
    fn binarize<R: GrammarRule>(&self, rule: &R, depth: usize) -> Self {
        let weight = if depth == 0 { self.weight } else { one() };
        WeightedHistory {
            inherit: self.inherit.binarize(rule, depth),
            weight,
        }
    }
}
