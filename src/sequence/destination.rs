//! Sequence destination.

use super::Sequence;

/// Trait for storing sequence rules in containers, with potential rewrites.
pub trait SequenceDestination {
    /// Inserts a sequence rule.
    fn add_sequence(&mut self, seq: Sequence);
}

impl<'a> SequenceDestination for &'a mut Vec<Sequence> {
    fn add_sequence(&mut self, seq: Sequence) {
        self.push(seq);
    }
}
