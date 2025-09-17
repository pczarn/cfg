//! Sequence destination.

use cfg_symbol::Symbol;

use super::{Sequence, builder::SequenceRuleBuilder};

/// Trait for storing sequence rules in containers, with potential rewrites.
pub trait SequenceDestination: Sized {
    /// Inserts a sequence rule.
    fn add_sequence(&mut self, seq: Sequence);

    /// Starts building a sequence rule.
    fn sequence(self, lhs: Symbol) -> SequenceRuleBuilder<Self> {
        SequenceRuleBuilder::new(self).sequence(lhs)
    }
}

impl<'a> SequenceDestination for &'a mut Vec<Sequence> {
    fn add_sequence(&mut self, seq: Sequence) {
        self.push(seq);
    }

    /// Starts building a sequence rule.
    fn sequence(self, lhs: Symbol) -> SequenceRuleBuilder<Self> {
        SequenceRuleBuilder::new(self).sequence(lhs)
    }
}
