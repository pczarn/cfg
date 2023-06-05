//! Sequence rules can be built with the builder pattern.

use std::ops::{Bound, RangeBounds};

use history::{HistorySource, NullHistorySource, RewriteSequence};
use sequence::destination::SequenceDestination;
use sequence::{Separator, Sequence};
use symbol::Symbol;

/// Sequence rule builder.
pub struct SequenceRuleBuilder<H, D, Hs = NullHistorySource> {
    lhs: Option<Symbol>,
    range: Option<(u32, Option<u32>)>,
    separator: Separator,
    history: Option<H>,
    history_state: Hs,
    destination: D,
}

impl<H, D> SequenceRuleBuilder<H, D>
where
    D: SequenceDestination<H>,
    H: RewriteSequence,
{
    /// Creates a sequence rule builder.
    pub fn new(destination: D) -> Self {
        SequenceRuleBuilder {
            lhs: None,
            range: None,
            history: None,
            history_state: NullHistorySource,
            separator: Separator::Null,
            destination: destination,
        }
    }
}

impl<H, D, Hs> SequenceRuleBuilder<H, D, Hs>
where
    D: SequenceDestination<H>,
    H: RewriteSequence,
{
    /// Sets the default history source.
    pub fn default_history<Hs2>(self, state: Hs2) -> SequenceRuleBuilder<H, D, Hs2> {
        SequenceRuleBuilder {
            lhs: self.lhs,
            range: self.range,
            history: self.history,
            history_state: state,
            separator: self.separator,
            destination: self.destination,
        }
    }

    /// Starts building a sequence rule.
    pub fn sequence(mut self, lhs: Symbol) -> Self {
        self.lhs = Some(lhs);
        self.separator = Separator::Null;
        self
    }

    /// Assigns the separator symbol and mode of separation.
    pub fn separator(mut self, sep: Separator) -> Self {
        self.separator = sep;
        self
    }

    /// Sets proper separation with the given separator symbol.
    pub fn intersperse(self, sym: Symbol) -> Self {
        self.separator(Separator::Proper(sym))
    }

    /// Assigns the rule history, which is used on the next call to `rhs`, or overwritten by a call
    /// to `rhs_with_history`.
    pub fn history(mut self, history: H) -> Self {
        self.history = Some(history);
        self
    }

    /// Assigns the inclusive range of the number of repetitions.
    pub fn inclusive(mut self, start: u32, end: Option<u32>) -> Self {
        self.range = Some((start, end));
        self
    }

    /// Adds a sequence rule to the grammar.
    pub fn rhs(mut self, rhs: Symbol) -> Self
    where
        Hs: HistorySource<H>,
    {
        let history = self.history.take().unwrap_or_else(|| {
            if let Some(sep) = self.separator.into() {
                self.history_state.build(self.lhs.unwrap(), &[rhs, sep])
            } else {
                self.history_state.build(self.lhs.unwrap(), &[rhs])
            }
        });
        self.rhs_with_history(rhs, history)
    }

    /// Adds a range to the sequence.
    pub fn range(self, range: impl RangeBounds<u32>) -> Self
    where
        Hs: HistorySource<H>,
        H: Default,
    {
        let to_option = |bound: Bound<u32>, diff| match bound {
            Bound::Included(included) => Some(included),
            Bound::Excluded(excluded) => Some((excluded as i64 + diff) as u32),
            Bound::Unbounded => None,
        };
        self.inclusive(
            to_option(range.start_bound().cloned(), 1).unwrap_or(0),
            to_option(range.end_bound().cloned(), -1),
        )
    }

    /// Adds a sequence rule to the grammar.
    pub fn rhs_with_range(self, rhs: Symbol, range: impl RangeBounds<u32>) -> Self
    where
        Hs: HistorySource<H>,
        H: Default,
    {
        self.range(range).rhs(rhs)
    }

    /// Adds a sequence rule to the grammar.
    pub fn rhs_with_history(mut self, rhs: Symbol, history: H) -> Self {
        let (start, end) = self.range.take().expect("expected inclusive(n, m)");
        self.destination.add_sequence(Sequence {
            lhs: self.lhs.unwrap(),
            rhs: rhs,
            start: start,
            end: end,
            separator: self.separator,
            history: history,
        });
        self
    }
}
