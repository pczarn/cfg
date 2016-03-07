//! Sequence destination.

use sequence::Sequence;

/// Trait for storing sequence rules in containers, with potential rewrites.
pub trait SequenceDestination<H> {
    /// Inserts a sequence rule.
    fn add_sequence(&mut self, seq: Sequence<H>);
}

impl<'a, H> SequenceDestination<H> for &'a mut Vec<Sequence<H>> {
    fn add_sequence(&mut self, seq: Sequence<H>) {
        self.push(seq);
    }
}
