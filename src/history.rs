//! Any data carried alongside a grammar rule can be its _history_. Rule histories may contain
//! more than semantic actions.

use std::marker::PhantomData;

use rule::GrammarRule;
use symbol::Symbol;

/// Used to inform which symbols on a rule'Symbol RHS are nullable, and will be eliminated.
#[derive(Clone, Copy)]
pub enum BinarizedRhsSubset {
    /// The first of two symbols.
    Left,
    /// The second of two symbols.
    Right,
    /// All 1 or 2 symbols. The rule is nullable.
    All,
}

/// A history which carries no data. All operations on `NullHistory` are no-op.
#[derive(Clone, Copy, Debug, Default)]
pub struct NullHistory;

/// Trait for history types that may have semantic actions.
pub trait Action {
    /// Returns a history with no-op semantic action.
    fn no_op(&self) -> Self;
}

/// Trait for history types that allow the rule to be binarized.
pub trait Binarize {
    /// Returns a history. May record the binarization.
    fn binarize<R>(&self, rule: &R, depth: usize) -> Self
    where
        R: GrammarRule;
}

/// Trait for history types that allow the rule to have nulling symbols
/// eliminated from the RHS.
pub trait EliminateNulling {
    /// Returns a history. May record the elimination.
    fn eliminate_nulling<R>(&self, rule: &R, which: BinarizedRhsSubset) -> Self
    where
        R: GrammarRule;
}

/// Trait for history types that allow the rule to have its precedence assigned.
pub trait AssignPrecedence {
    /// Returns a history. May record the precedence.
    fn assign_precedence<R>(&self, rule: &R, looseness: u32) -> Self
    where
        R: GrammarRule;
}

/// Trait for history types that allow the sequence rule to be rewritten into grammar rules.
pub trait RewriteSequence {
    /// Must be an `Action`, because all created grammar rules except the topmost one will have
    /// no-op semantic action.
    type Rewritten;

    /// Returns a history. May record the rewrite.
    fn top(&self, rhs: Symbol, sep: Option<Symbol>, new_rhs: &[Symbol]) -> Self::Rewritten;
    /// Returns a history. May record the rewrite.
    fn bottom(&self, rhs: Symbol, sep: Option<Symbol>, new_rhs: &[Symbol]) -> Self::Rewritten;
}

impl Action for NullHistory {
    fn no_op(&self) -> Self {
        NullHistory
    }
}

impl Binarize for NullHistory {
    fn binarize<R>(&self, _rule: &R, _depth: usize) -> Self {
        NullHistory
    }
}

impl EliminateNulling for NullHistory {
    fn eliminate_nulling<R>(&self, _rule: &R, _which: BinarizedRhsSubset) -> Self {
        NullHistory
    }
}

impl AssignPrecedence for NullHistory {
    fn assign_precedence<R>(&self, _rule: &R, _looseness: u32) -> Self {
        NullHistory
    }
}

impl RewriteSequence for NullHistory {
    type Rewritten = Self;

    fn top(&self, _rhs: Symbol, _sep: Option<Symbol>, _new: &[Symbol]) -> Self {
        NullHistory
    }

    fn bottom(&self, _rhs: Symbol, _sep: Option<Symbol>, _new: &[Symbol]) -> Self {
        NullHistory
    }
}

impl<'a, T> RewriteSequence for &'a T
where
    T: RewriteSequence,
{
    type Rewritten = T::Rewritten;

    fn top(&self, rhs: Symbol, sep: Option<Symbol>, new_rhs: &[Symbol]) -> Self::Rewritten {
        (**self).top(rhs, sep, new_rhs)
    }

    fn bottom(&self, rhs: Symbol, sep: Option<Symbol>, new_rhs: &[Symbol]) -> Self::Rewritten {
        (**self).bottom(rhs, sep, new_rhs)
    }
}

/// A trait for history factories.
pub trait HistorySource<H> {
    /// Create a history.
    fn build(&mut self, lhs: Symbol, rhs: &[Symbol]) -> H;
}

/// Clone history.
pub struct CloneHistory<'a, H: 'a> {
    history: &'a H,
    marker: PhantomData<Symbol>,
}

impl<'a, H> CloneHistory<'a, H> {
    /// Creates a cloned history factory.
    pub fn new(history: &'a H) -> Self {
        CloneHistory {
            history: history,
            marker: PhantomData,
        }
    }
}

impl<'a, H> HistorySource<H> for CloneHistory<'a, H>
where
    H: Clone,
{
    fn build(&mut self, _lhs: Symbol, _rhs: &[Symbol]) -> H {
        self.history.clone()
    }
}

/// Factory of default histories.
pub struct DefaultHistory<H>(PhantomData<H>);

impl<H> DefaultHistory<H> {
    /// Creates a default history factory.
    pub fn new() -> Self {
        DefaultHistory(PhantomData)
    }
}

impl<H> HistorySource<H> for DefaultHistory<H>
where
    H: Default,
{
    fn build(&mut self, _lhs: Symbol, _rhs: &[Symbol]) -> H {
        H::default()
    }
}

/// A source that only works for building NullHistory.
#[derive(Clone, Copy)]
pub struct NullHistorySource;

impl HistorySource<NullHistory> for NullHistorySource {
    fn build(&mut self, _lhs: Symbol, _rhs: &[Symbol]) -> NullHistory {
        NullHistory
    }
}
