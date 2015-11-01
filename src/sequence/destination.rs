//! Sequence destination.

use sequence::Sequence;
use symbol::GrammarSymbol;

/// Trait for storing sequence rules in containers, with potential rewrites.
pub trait SequenceDestination<H> {
    /// The type of symbols.
    type Symbol;
    /// Inserts a sequence rule.
    fn add_sequence(&mut self, seq: Sequence<H, Self::Symbol>);
}

impl<'a, H, S> SequenceDestination<H> for &'a mut Vec<Sequence<H, S>> where S: GrammarSymbol {
    type Symbol = S;

    fn add_sequence(&mut self, seq: Sequence<H, Self::Symbol>) {
        self.push(seq);
    }
}
