//! Sequence rules can be built with the builder pattern.

#[cfg(feature = "nightly")]
use collections::range::RangeArgument;

use history::{RewriteSequence, NullHistorySource, HistorySource};
use sequence::{Separator, Sequence};
use sequence::destination::SequenceDestination;
use symbol::GrammarSymbol;

/// Sequence rule builder.
pub struct SequenceRuleBuilder<H, D, S, Hs = NullHistorySource>
    where S: GrammarSymbol
{
    lhs: Option<S>,
    range: Option<(u32, Option<u32>)>,
    separator: Separator<S>,
    history: Option<H>,
    history_state: Hs,
    destination: D,
}

impl<H, D, S> SequenceRuleBuilder<H, D, S>
    where D: SequenceDestination<H, Symbol = S>,
          H: RewriteSequence,
          S: GrammarSymbol
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

impl<H, D, S, Hs> SequenceRuleBuilder<H, D, S, Hs>
    where D: SequenceDestination<H, Symbol = S>,
          H: RewriteSequence,
          S: GrammarSymbol
{
    /// Sets the default history source.
    pub fn default_history<Hs2>(self, state: Hs2) -> SequenceRuleBuilder<H, D, S, Hs2> {
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
    pub fn sequence(mut self, lhs: S) -> Self {
        self.lhs = Some(lhs);
        self
    }

    /// Assigns the separator symbol and mode of separation.
    pub fn separator(mut self, sep: Separator<S>) -> Self {
        self.separator = sep;
        self
    }

    /// Sets proper separation with the given separator symbol.
    pub fn intersperse(self, sym: S) -> Self {
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
    pub fn rhs(mut self, rhs: S) -> Self
        where Hs: HistorySource<H, S>
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

    /// Adds a sequence rule to the grammar.
    #[cfg(feature = "nightly")]
    pub fn rhs_with_range<T>(mut self, rhs: S, range: T) -> Self
        where T: RangeArgument<u32>,
              H: Default
    {
        let history = self.history.take();
        self.inclusive(range.start().cloned().unwrap_or(0),
                       range.end().cloned().map(|end| end - 1))
            .rhs(rhs)
    }

    /// Adds a sequence rule to the grammar.
    pub fn rhs_with_history(mut self, rhs: S, history: H) -> Self {
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
