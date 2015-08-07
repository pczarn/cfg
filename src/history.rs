//! Any data carried alongside a grammar rule can be its _history_. Rule histories may contain
//! more than semantic actions.

use rule::GrammarRule;
use sequence::Sequence;
use symbol::GrammarSymbol;

/// Used to inform which symbols on a rule's RHS are nullable, and will be eliminated.
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
#[derive(Clone, Debug, Default)]
pub struct NullHistory;

/// Trait for history types that may have semantic actions.
pub trait Action {
    /// Returns a history with no-op semantic action.
    fn no_op(&self) -> Self;
}

/// Trait for history types that allow the rule to be binarized.
pub trait Binarize {
    /// Returns a history. May record the binarization.
    fn binarize<R>(&self, rule: &R, depth: usize) -> Self where R: GrammarRule;
}

/// Trait for history types that allow the rule to have nulling symbols
/// eliminated from the RHS.
pub trait EliminateNulling {
    /// Returns a history. May record the elimination.
    fn eliminate_nulling<R>(&self, rule: &R, which: BinarizedRhsSubset) -> Self where
            R: GrammarRule;
}

/// Trait for history types that allow the rule to have its precedence assigned.
pub trait AssignPrecedence {
    /// Returns a history. May record the precedence.
    fn assign_precedence<R>(&self, rule: &R, looseness: u32) -> Self where R: GrammarRule;
}

/// Trait for history types that allow the sequence rule to be rewritten into grammar rules.
pub trait RewriteSequence {
    /// Must be an `Action`, because all created grammar rules except the topmost one will have
    /// no-op semantic action.
    type Rewritten: Action;

    /// Returns a history. May record the rewrite.
    fn sequence<H, S>(&self, top: &Sequence<H, S>) -> Self::Rewritten where S: GrammarSymbol;
}

impl Action for NullHistory {
    fn no_op(&self) -> Self { NullHistory }
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

    fn sequence<H, S>(&self, _top: &Sequence<H, S>) -> Self {
        NullHistory
    }
}

impl<'a, T> RewriteSequence for &'a T where T: RewriteSequence {
    type Rewritten = T::Rewritten;

    fn sequence<H, S>(&self, top: &Sequence<H, S>) -> Self::Rewritten
            where S: GrammarSymbol {
        (**self).sequence(top)
    }
}
