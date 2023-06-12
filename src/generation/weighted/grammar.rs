use std::ops::{Deref, DerefMut};

use super::history::{
    BuildWeightedHistory, BuildWeightedSequenceHistory, WeightedHistory, WeightedSequenceHistory,
};
use super::*;
use crate::rule::builder::RuleBuilder;
use crate::sequence::builder::SequenceRuleBuilder;
use crate::sequence::Sequence;
use crate::Symbol;
use crate::{Cfg, ContextFree, ContextFreeRef};

/// Drop-in replacement for `cfg::Cfg` that traces relations between user-provided
/// and internal grammars.
#[derive(Default)]
pub struct WeightedGrammar<W> {
    inherit: Cfg<WeightedHistory<W>, WeightedSequenceHistory<W>>,
    start: Option<Symbol>,
}

impl<W: Weight> WeightedGrammar<W> {
    pub fn new() -> Self {
        WeightedGrammar {
            inherit: Cfg::new(),
            start: None,
        }
    }

    pub fn set_start(&mut self, start: Symbol) {
        self.start = Some(start);
    }

    pub fn start(&self) -> Symbol {
        self.start.unwrap()
    }

    pub fn rule(
        &mut self,
        lhs: Symbol,
    ) -> RuleBuilder<&mut Cfg<WeightedHistory<W>, WeightedSequenceHistory<W>>, BuildWeightedHistory>
    {
        let rule_count = self.inherit.rules().count() + self.sequence_rules().len();
        self.inherit
            .rule(lhs)
            .default_history(BuildWeightedHistory::new(rule_count))
    }

    pub fn sequence(
        &mut self,
        lhs: Symbol,
    ) -> SequenceRuleBuilder<
        WeightedSequenceHistory<W>,
        &mut Vec<Sequence<WeightedSequenceHistory<W>>>,
        BuildWeightedSequenceHistory,
    > {
        let rule_count = self.inherit.rules().count() + self.sequence_rules().len();
        self.inherit
            .sequence(lhs)
            .default_history(BuildWeightedSequenceHistory::new(rule_count))
    }

    pub fn binarize(&self) -> WeightedBinarizedGrammar<W> {
        WeightedBinarizedGrammar {
            inherit: self.inherit.binarize(),
            start: self.start,
        }
    }
}

impl<W> Deref for WeightedGrammar<W> {
    type Target = Cfg<WeightedHistory<W>, WeightedSequenceHistory<W>>;
    fn deref(&self) -> &Self::Target {
        &self.inherit
    }
}

impl<W> DerefMut for WeightedGrammar<W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inherit
    }
}
