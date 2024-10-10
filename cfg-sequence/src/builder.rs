//! Sequence rules can be built with the builder pattern.

use std::ops::{Bound, RangeBounds};

use crate::destination::SequenceDestination;
use crate::{Separator, Sequence};
use cfg_history::HistoryId;
use cfg_symbol::Symbol;

/// Sequence rule builder.
pub struct SequenceRuleBuilder<D: SequenceDestination> {
    lhs: Option<Symbol>,
    range: Option<(u32, Option<u32>)>,
    separator: Separator,
    history: Option<HistoryId>,
    default_history: Option<HistoryId>,
    destination: D,
}

impl<D> SequenceRuleBuilder<D>
where
    D: SequenceDestination,
{
    /// Creates a sequence rule builder.
    pub fn new(destination: D) -> Self {
        SequenceRuleBuilder {
            lhs: None,
            range: None,
            history: None,
            default_history: None,
            separator: Separator::Null,
            destination,
        }
    }

    /// Sets the default history source.
    pub fn default_history(self, default_history: HistoryId) -> Self {
        SequenceRuleBuilder {
            lhs: self.lhs,
            range: self.range,
            history: self.history,
            default_history: Some(default_history),
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
    pub fn history(mut self, history: HistoryId) -> Self {
        self.history = Some(history);
        self
    }

    /// Assigns the inclusive range of the number of repetitions.
    pub fn inclusive(mut self, start: u32, end: Option<u32>) -> Self {
        self.range = Some((start, end));
        self
    }

    /// Adds a sequence rule to the grammar.
    pub fn rhs(mut self, rhs: Symbol) -> Self {
        let history = self.history.take().or(self.default_history);
        self.rhs_with_history(rhs, history)
    }

    /// Adds a range to the sequence.
    pub fn range(self, range: impl RangeBounds<u32>) -> Self {
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
    pub fn rhs_with_range(self, rhs: Symbol, range: impl RangeBounds<u32>) -> Self {
        self.range(range).rhs(rhs)
    }

    /// Adds a sequence rule to the grammar.
    pub fn rhs_with_history(mut self, rhs: Symbol, history_id: Option<HistoryId>) -> Self {
        let (start, end) = self.range.take().expect("expected inclusive(n, m)");
        self.destination.add_sequence(Sequence {
            lhs: self.lhs.unwrap(),
            rhs,
            start,
            end,
            separator: self.separator,
            history_id,
        });
        self
    }
}
